    mod monaco;
pub mod diffsync;
pub mod remote_cursors;


use crate::code::monaco::{ EditorSelection, EditorPosition };
use lighthousemc_editor_common::packet::c2s::{ SelectionsC2SPacket, SelectionRange };
use std::sync::atomic::Ordering;
use wasm_bindgen::prelude::*;
use web_sys::KeyboardEvent;


pub fn init() {
    let window   = web_sys::window().unwrap();
    let document = window.document().unwrap();

    monaco::init_theme();

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


    let keydown_callback = Closure::<dyn FnMut(_) -> ()>::new(move |event : KeyboardEvent| {
        match ((event.ctrl_key(), event.alt_key(), event.key().as_str())) {

            (true, false, "r") => { event.prevent_default(); },

            (true, false, "s") => { event.prevent_default(); },

            (true, false, "f") => { event.prevent_default(); },

            (true, false, "w") => {
                event.prevent_default();
                if let Some((file_id, file_path)) = crate::filetabs::currently_focused() {
                    crate::state::close_file(file_id, Some(file_path));
                }
            },

            (true, false, "W") => {
                event.prevent_default();
                for (file_id, file_path) in crate::filetabs::list_all() {
                    crate::state::close_file(file_id, Some(file_path));
                }
            },

            (true, false, "T") => {
                event.prevent_default();
                crate::state::reopen_history();
            },

            _ => { }
        }
    });
    document.add_event_listener_with_callback("keydown", keydown_callback.as_ref().unchecked_ref()).unwrap();
    keydown_callback.forget();

}


pub fn open_noopen() { open("editor_right_main_noopen"); }

pub fn open_nontext() { open("editor_right_main_nontext"); }

pub fn open_load() { open("editor_right_main_loader"); }

pub fn create_monaco(file_id : u64, file_name : &str, initial_script : &str, open : bool) {
    if (open) { close(); }
    monaco::create(file_id, file_name, initial_script.to_string(), open);
}
pub fn open_monaco(file_id : u64) {
    close();
    monaco::open(file_id);
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


pub fn destroy_monaco(file_id : u64) {
    monaco::destroy(file_id);
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
