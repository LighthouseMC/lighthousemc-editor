use voxidian_editor_common::packet::s2c::FileTreeEntry;
use voxidian_editor_common::packet::c2s::*;
use std::cell::LazyCell;
use std::sync::{ RwLock, RwLockReadGuard, RwLockWriteGuard, Mutex };
use std::collections::{ HashMap, VecDeque };


pub static FILES : FilesContainer = FilesContainer::new();
pub struct FilesContainer {
    files : LazyCell<RwLock<HashMap<u64, FilesEntry>>>
}
impl FilesContainer { const fn new() -> Self { Self {
    files : LazyCell::new(|| RwLock::new(HashMap::new()))
} } }
impl FilesContainer {
    pub fn read(&self) -> RwLockReadGuard<HashMap<u64, FilesEntry>> {
        self.files.read().unwrap()
    }
    pub fn write(&self) -> RwLockWriteGuard<HashMap<u64, FilesEntry>> {
        self.files.write().unwrap()
    }
}
unsafe impl Sync for FilesContainer { }


static FILE_HISTORY : Mutex<VecDeque<(u64, String)>> = Mutex::new(VecDeque::new());


#[derive(Debug)]
pub struct FilesEntry {
    pub file_id : u64,
    pub fsname  : String,
    pub is_open : Option<Option<FilesEntryContents>>
    //            |      |      ^- File data
    //            |      ^- None if opened but no data from server yet
    //            ^- None if file not opened
}
#[derive(Debug)]
pub enum FilesEntryContents {
    NonText,
    Text(String)
}


pub fn add_tree_entry(entry : FileTreeEntry<'static>) {
    if (! entry.is_dir) {
        FILES.write().insert(entry.entry_id, FilesEntry {
            file_id : entry.entry_id,
            fsname  : entry.fsname.to_string(),
            is_open : None
        });
    }
    crate::filetree::add(entry);
}

pub fn open_file(file_id : u64, path : String, remove_history : bool) -> bool {
    if (remove_history) {
        FILE_HISTORY.lock().unwrap().retain(|(file, _)| *file != file_id);
    }
    let mut files = FILES.write();
    let Some(FilesEntry { is_open, .. }) = files.get_mut(&file_id) else { return false; };
    crate::code::remote_cursors::update();
    let mut should_proper_update_cursor = true;
    match (is_open) {
        Some(Some(FilesEntryContents::NonText)) => { crate::code::open_nontext(); },
        Some(Some(FilesEntryContents::Text(_))) => { crate::code::open_monaco(file_id); },
        Some(None) => { crate::code::open_load(); },
        None => {
            *is_open = Some(None);
            crate::ws::WS.send(OpenFileC2SPacket { file_id });
            crate::code::open_load();
            should_proper_update_cursor = false;
        }
    }
    crate::filetree::open_file(file_id);
    crate::filetabs::open_file(file_id, path);
    if (should_proper_update_cursor) {
        crate::code::selection_changed();
    } else {
        crate::code::selection_unchanged();
        crate::ws::WS.send(SelectionsC2SPacket {
            selections : Some((file_id, vec![ SelectionRange {
                start : 0,
                end   : 0
            } ]))
        });
    }
    true
}

pub fn close_file(file_id : u64, history_path : Option<String>) {
    if let Some(history_path) = history_path {
        FILE_HISTORY.lock().unwrap().push_back((file_id, history_path));
    }
    let mut files = FILES.write();
    let Some(FilesEntry { is_open, .. }) = files.get_mut(&file_id) else { return; };
    if let Some(_) = is_open {
        *is_open = None;
        drop(files);
        crate::filetabs::close(file_id);
        crate::code::selection_changed();
        crate::code::remote_cursors::update();
        crate::ws::WS.send(CloseFileC2SPacket { file_id });
    }
}


pub fn reopen_history() {
    let mut history = FILE_HISTORY.lock().unwrap();
    loop {
        let Some((file_id, file_path)) = history.pop_back() else { break };
        if (open_file(file_id, file_path, false)) { break; }
    }
}
