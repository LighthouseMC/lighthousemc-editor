use voxidian_editor_common::packet::s2c::{ FileTreeEntry, FileContents };
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
    pub id      : u64,
    pub fsname  : String,
    pub is_open : Option<Option<FileContents>>
    //            |      |      ^- File data
    //            |      ^- None if opened but no data from server yet
    //            ^- None if file not opened
}


pub fn add_tree_entry(entry : FileTreeEntry) {
    if (! entry.is_dir) {
        FILES.write().insert(entry.id, FilesEntry {
            id      : entry.id,
            fsname  : entry.fsname.clone(),
            is_open : None
        });
    }
    crate::filetree::add(entry);
}

pub fn open_file(id : u64, path : String, remove_history : bool) -> bool {
    if (remove_history) {
        FILE_HISTORY.lock().unwrap().retain(|(file, _)| *file != id);
    }
    let mut files = FILES.write();
    let Some(FilesEntry { is_open, .. }) = files.get_mut(&id) else { return false; };
    crate::code::remote_cursors::update();
    let mut should_proper_update_cursor = true;
    match (is_open) {
        Some(Some(FileContents::NonText)) => { crate::code::open_nontext(); },
        Some(Some(FileContents::Text(_))) => { crate::code::open_monaco(id); },
        Some(None) => { crate::code::open_load(); },
        None => {
            *is_open = Some(None);
            crate::ws::WS.send(OpenFileC2SPacket { id });
            crate::code::open_load();
            should_proper_update_cursor = false;
        }
    }
    crate::filetree::open_file(id);
    crate::filetabs::open_file(id, path);
    if (should_proper_update_cursor) {
        crate::code::selection_changed();
    } else {
        crate::code::selection_unchanged();
        crate::ws::WS.send(SelectionsC2SPacket {
            selections : Some((id, vec![ SelectionRange {
                start : 0,
                end   : 0
            } ]))
        });
    }
    true
}

pub fn close_file(id : u64, path : String) {
    FILE_HISTORY.lock().unwrap().push_back((id, path));
    let mut files = FILES.write();
    let Some(FilesEntry { is_open, .. }) = files.get_mut(&id) else { return; };
    if let Some(_) = is_open {
        *is_open = None;
        drop(files);
        crate::filetabs::close(id);
        crate::code::selection_changed();
        crate::code::remote_cursors::update();
        crate::ws::WS.send(CloseFileC2SPacket { id });
    }
}


pub fn reopen_history() {
    let mut history = FILE_HISTORY.lock().unwrap();
    loop {
        let Some((file_id, file_path)) = history.pop_back() else { break };
        if (open_file(file_id, file_path, false)) { break; }
    }
}
