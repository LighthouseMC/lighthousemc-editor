use crate::code::monaco::{ self, Editor, EditorDecoration, EditorDecorationOptions, EditorSelection, EditorHoverMessage };
use voxidian_editor_common::packet::c2s::SelectionRange;
use std::sync::atomic::AtomicBool;
use std::sync::{ RwLock, RwLockReadGuard, RwLockWriteGuard, Mutex };
use std::cell::LazyCell;
use std::collections::HashMap;


pub fn init_css() {
    let window   = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let     head  = document.get_element_by_id("head").unwrap();
    let     style = document.create_element("style").unwrap();
    let mut inner = String::new();

    for hue in 0..360 {
        inner += &format!(".editor_code_remote_selection_{} {{ background: hsl({},100%,62.5%); opacity: 0.625; min-width: 2px; }}", hue, hue);
    }

    style.set_inner_html(&inner);
    head.append_child(&style).unwrap();
}


pub(super) static SELECTION_CHANGED : AtomicBool = AtomicBool::new(false);


pub(crate) static REMOTE_SELECTIONS : RemoteSelectionsContainer = RemoteSelectionsContainer::new();
pub(crate) struct RemoteSelectionsContainer {
    selections      : LazyCell<RwLock<HashMap<u64, RemoteSelection>>>,
    old_decorations : Mutex<Option<Vec<String>>>
}
#[derive(Debug)]
pub(crate) struct RemoteSelection {
    pub client_name : String,
    pub colour      : u16,
    pub file_id     : u32,
    pub selections  : Vec<SelectionRange>
}
impl RemoteSelectionsContainer { const fn new() -> Self { Self {
    selections      : LazyCell::new(|| RwLock::new(HashMap::new())),
    old_decorations : Mutex::new(Some(Vec::new()))
} } }
impl RemoteSelectionsContainer {
    pub(crate) fn read(&self) -> RwLockReadGuard<HashMap<u64, RemoteSelection>> {
        self.selections.read().unwrap()
    }
    pub(crate) fn write(&self) -> RwLockWriteGuard<HashMap<u64, RemoteSelection>> {
        self.selections.write().unwrap()
    }
}
unsafe impl Sync for RemoteSelectionsContainer { }


pub(crate) fn update() {
    let editors = monaco::EDITORS.read();
    if let Some((file_id, editor)) = monaco::currently_focused().and_then(|currently_focused| editors.get(&currently_focused).map(|editor| (currently_focused, editor))) {
        update_known(file_id, editor);
    }
}


pub(crate) fn update_known(file_id : u32, editor : &Editor) {
    let mut new_decorations = Vec::new();

    for (_, remote_selection) in &*REMOTE_SELECTIONS.read() {
        if (remote_selection.file_id == file_id) {
            for selection in &remote_selection.selections {
                new_decorations.push(serde_wasm_bindgen::to_value(&EditorDecoration {
                    options : EditorDecorationOptions {
                        class_name    : format!("editor_code_remote_selection_{}", remote_selection.colour),
                        hover_message : EditorHoverMessage { value : remote_selection.client_name.clone() },
                        is_whole_line : false,
                        stickiness    : 1
                    },
                    range   : EditorSelection {
                        start_line   : selection.start_line,
                        start_column : selection.start_column,
                        end_line     : selection.end_line,
                        end_column   : selection.end_column
                    }
                }).unwrap());
            }
        }
    }

    let model = editor.get_model();
    let mut old_decorations = REMOTE_SELECTIONS.old_decorations.lock().unwrap();
    *old_decorations = Some(model.delta_decorations(old_decorations.take().unwrap(), new_decorations));
}
