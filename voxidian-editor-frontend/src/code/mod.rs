    mod monaco;
pub mod diffsync;
pub mod remote_cursors;


use crate::code::monaco::{ EditorSelection, EditorPosition };
use voxidian_editor_common::packet::c2s::{ SelectionsC2SPacket, SelectionRange };
use std::sync::atomic::Ordering;
use wasm_bindgen::prelude::*;


pub fn init() {

    remote_cursors::init_css();

    let timeout_callback = Closure::<dyn FnMut() -> ()>::new(move || {
        if (remote_cursors::SELECTION_CHANGED.swap(false, Ordering::Relaxed)) {
            let editors = monaco::EDITORS.read();
            let selections = monaco::currently_focused().and_then(|currently_focused| editors.get(&currently_focused).map(|editor| (currently_focused, editor))).map(|(id, editor)| {
                let model = editor.get_model();
                (id, editor.get_selections().into_iter().map(|selection| {
                    let selection = serde_wasm_bindgen::from_value::<EditorSelection>(selection).unwrap();
                    SelectionRange {
                        start : model.get_offset_at(serde_wasm_bindgen::to_value(&EditorPosition { line : selection.start_line , column : selection.start_column }).unwrap()),
                        end   : model.get_offset_at(serde_wasm_bindgen::to_value(&EditorPosition { line : selection.end_line   , column : selection.end_column   }).unwrap())
                    }
                }).collect::<Vec<_>>())
            });
            crate::ws::WS.send(SelectionsC2SPacket {
                selections
            });
        }
    });
    crate::set_interval(timeout_callback.as_ref().unchecked_ref(), 250);
    timeout_callback.forget();

}


pub fn open_noopen() { open("editor_right_main_noopen"); }

pub fn open_nontext() { open("editor_right_main_nontext"); }

pub fn open_load() { open("editor_right_main_loader"); }

pub fn create_monaco(id : u32, initial_script : &str, open : bool) {
    if (open) { close(); }
    monaco::create(id, initial_script.to_string(), "text".to_string(), open);
}
pub fn open_monaco(id : u32) {
    close();
    monaco::open(id);
}

fn open(selected : &str) {
    let window    = web_sys::window().unwrap();
    let document  = window.document().unwrap();
    let container = document.get_element_by_id("editor_right_main_container").unwrap();
    let children  = container.children();
    for i in 0..children.length() {
        let child = children.get_with_index(i).unwrap();
        child.class_list().toggle_with_force("editor_right_main_selected", child.id() == selected).unwrap();
    }
}


pub fn destroy_monaco(id : u32) {
    monaco::destroy(id);
}

fn close() {
    let window    = web_sys::window().unwrap();
    let document  = window.document().unwrap();
    let container = document.get_element_by_id("editor_right_main_container").unwrap();
    let children  = container.children();
    for i in 0..children.length() {
        let child = children.get_with_index(i).unwrap();
        child.class_list().toggle_with_force("editor_right_main_selected", false).unwrap();
    }
}


pub fn selection_changed() {
    remote_cursors::SELECTION_CHANGED.store(true, Ordering::Relaxed);
}
pub fn selection_unchanged() {
    remote_cursors::SELECTION_CHANGED.store(false, Ordering::Relaxed);
}
