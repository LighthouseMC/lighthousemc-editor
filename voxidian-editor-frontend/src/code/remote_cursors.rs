use crate::code::monaco::{ self, Editor, EditorDecoration, EditorDecorationOptions, EditorSelection, EditorHoverMessage, EditorPosition };
use voxidian_editor_common::packet::c2s::SelectionRange;
use std::sync::atomic::AtomicBool;
use std::sync::{ RwLock, RwLockReadGuard, RwLockWriteGuard, Mutex };
use std::cell::LazyCell;
use std::collections::HashMap;
use web_sys::Element;


pub fn init_css() {
    let window   = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let     head  = document.get_element_by_id("head").unwrap();
    let     style = document.create_element("style").unwrap();
    let mut inner = String::new();

    for hue in 0..360 {
        inner += &format!(".editor_code_remote_selection_{}_single {{ background: hsl({},100%,62.5%); opacity: 0.875; min-width: 2px; }}", hue, hue);
        inner += &format!(".editor_code_remote_selection_{}_range {{ background: hsl({},100%,62.5%); opacity: 0.5; min-width: 2px; }}", hue, hue);
    }

    style.set_inner_html(&inner);
    head.append_child(&style).unwrap();
}


pub(super) static SELECTION_CHANGED : AtomicBool = AtomicBool::new(false);


pub(crate) static REMOTE_SELECTIONS : RemoteSelectionsContainer = RemoteSelectionsContainer::new();
pub(crate) struct RemoteSelectionsContainer {
    selections           : LazyCell<RwLock<HashMap<u64, RemoteSelection>>>,
    old_decorations      : Mutex<Option<Vec<String>>>,
    old_filetree_markers : Mutex<Vec<Element>>
}
#[derive(Debug)]
pub(crate) struct RemoteSelection {
    pub client_name : String,
    pub colour      : u16,
    pub file_id     : u64,
    pub selections  : Vec<SelectionRange>
}
impl RemoteSelectionsContainer { const fn new() -> Self { Self {
    selections           : LazyCell::new(|| RwLock::new(HashMap::new())),
    old_decorations      : Mutex::new(Some(Vec::new())),
    old_filetree_markers : Mutex::new(Vec::new())
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
    let window   = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let editors = monaco::EDITORS.read();
    if let Some((file_id, editor)) = monaco::currently_focused().and_then(|currently_focused| editors.get(&currently_focused).map(|editor| (currently_focused, editor))) {
        update_known(file_id, editor);
    }

    let mut old_filetree_markers = REMOTE_SELECTIONS.old_filetree_markers.lock().unwrap();
    // Clear previous markers in file tree.
    for marker in old_filetree_markers.drain(..) {
        marker.remove();
    }
    // Add new markers.
    let filetree_entries  = document.get_elements_by_class_name("editor_filetree_file");
    let remote_selections = REMOTE_SELECTIONS.read();
    for i in 0..filetree_entries.length() {
        let filetree_entry = filetree_entries.get_with_index(i).unwrap();
        let file_id = filetree_entry.get_attribute("editor_filetree_file_id").unwrap().parse::<u64>().unwrap();
        let mut markers = Vec::new();
        for (client_id, remote_selection) in &*remote_selections {
            if (remote_selection.file_id == file_id) {
                markers.push((client_id, remote_selection.colour));
            }
        }
        markers.sort_by_key(|(client_id, _)| **client_id);
        for (_, colour) in markers {
            let marker = document.create_element("div").unwrap();
            let marker_classes = marker.class_list();
            marker_classes.toggle_with_force("editor_filetree_entry_remote", true).unwrap();
            marker_classes.toggle_with_force(&format!("editor_code_remote_selection_{}_single", colour), true).unwrap();
            filetree_entry.append_child(&marker).unwrap();
            old_filetree_markers.push(marker);
        }
    }
}


pub(crate) fn update_known(file_id : u64, editor : &Editor) {

    // Add markers in the file.
    let model = editor.get_model();
    let mut new_decorations = Vec::new();
    for (_, remote_selection) in &*REMOTE_SELECTIONS.read() {
        if (remote_selection.file_id == file_id) {
            for selection in &remote_selection.selections {
                let start = serde_wasm_bindgen::from_value::<EditorPosition>(model.get_position_at(selection.start )).unwrap();
                let end   = serde_wasm_bindgen::from_value::<EditorPosition>(model.get_position_at(selection.end   )).unwrap();
                new_decorations.push(serde_wasm_bindgen::to_value(&EditorDecoration {
                    options : EditorDecorationOptions {
                        class_name    : format!("editor_code_remote_selection_{}_{}", remote_selection.colour, if (selection.start == selection.end) { "single" } else { "range" }).into(),
                        hover_message : EditorHoverMessage { value : (&remote_selection.client_name).into() },
                        is_whole_line : false,
                        stickiness    : 1
                    },
                    range   : EditorSelection {
                        start_line   : start.line,
                        start_column : start.column,
                        end_line     : end.line,
                        end_column   : end.column
                    }
                }).unwrap());
            }
        }
    }
    let mut old_decorations = REMOTE_SELECTIONS.old_decorations.lock().unwrap();
    *old_decorations = Some(model.delta_decorations(old_decorations.take().unwrap(), new_decorations));
}
