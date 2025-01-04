use voxidian_editor_common::packet::s2c::{ FileTreeEntry, FileContents };
use voxidian_editor_common::packet::c2s::*;
use std::cell::LazyCell;
use std::sync::{ RwLock, RwLockReadGuard, RwLockWriteGuard };
use std::collections::HashMap;


pub static FILES : FilesContainer = FilesContainer::new();
pub struct FilesContainer {
    files : LazyCell<RwLock<HashMap<u32, FilesEntry>>>
}
impl FilesContainer { const fn new() -> Self { Self {
    files : LazyCell::new(|| RwLock::new(HashMap::new()))
} } }
impl FilesContainer {
    pub fn read(&self) -> RwLockReadGuard<HashMap<u32, FilesEntry>> {
        self.files.read().unwrap()
    }
    pub fn write(&self) -> RwLockWriteGuard<HashMap<u32, FilesEntry>> {
        self.files.write().unwrap()
    }
}
unsafe impl Sync for FilesContainer { }


#[derive(Debug)]
pub struct FilesEntry {
    pub id   : u32,
    pub path : String,
    pub kind : FilesEntryKind
}
#[derive(Debug)]
pub enum FilesEntryKind {
    Directory,
    File {
        is_open : Option<Option<FileContents>>
        //        |      |      ^- File data
        //        |      ^- None if opened but no data from server yet
        //        ^- None if file not opened
    }
}


pub fn add_file(entry : &FileTreeEntry) {
    FILES.write().insert(entry.id, FilesEntry {
        id   : entry.id,
        path : entry.path.clone(),
        kind : if (entry.is_dir) { FilesEntryKind::Directory }
        else { FilesEntryKind::File {
            is_open : None
        } }
    });
    crate::filetree::add(&entry);
}

pub fn open_file(id : u32) {
    let mut files = FILES.write();
    let Some(FilesEntry { path, kind : FilesEntryKind::File { is_open }, .. }) = files.get_mut(&id) else { return; };
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
    crate::filetree::open(&path);
    crate::filetabs::open(id, &path);
    if (should_proper_update_cursor) {
        crate::code::selection_changed();
    } else {
        crate::code::selection_unchanged();
        crate::ws::WS.send(SelectionsC2SPacket {
            selections : Some((id, vec![ SelectionRange {
                start_line   : 1,
                start_column : 1,
                end_line     : 1,
                end_column   : 1
            } ]))
        });
    }
}

pub fn close_file(id : u32) {
    let mut files = FILES.write();
    let Some(FilesEntry { path, kind : FilesEntryKind::File { is_open }, .. }) = files.get_mut(&id) else { return; };
    if let Some(_) = is_open {
        *is_open = None;
        let path = path.clone();
        drop(files);
        crate::filetabs::close(&path);
        crate::code::selection_changed();
        crate::code::remote_cursors::update();
        crate::ws::WS.send(CloseFileC2SPacket { id });
    }
}
