use crate::state::{ FilesEntry, FilesEntryContents };
use lighthousemc_editor_common::packet::s2c::FileContents;
use wasm_bindgen::prelude::*;
use web_sys::PointerEvent;


pub fn open_file(file_id : u64, path : String) {
    let window   = web_sys::window().unwrap();
    let document = window.document().unwrap();

    // File tab
    let filetabs = document.get_element_by_id("editor_filetabs").unwrap();
    let children = filetabs.children();
    let mut before = None;
    let mut found  = None;
    for i in 0..children.length() {
        let tab = children.get_with_index(i).unwrap();
        if (tab.id() == "editor_filetab_selected") {
            before = tab.next_sibling();
            tab.remove_attribute("id").unwrap();
        }
        let other_file_id = tab.get_attribute("editor_filetab_file_id").unwrap().parse::<u64>().unwrap();
        if (other_file_id == file_id) {
            found = Some(tab);
        }
    }
    found.unwrap_or_else(|| {
        let div = document.create_element("div").unwrap();
        div.set_attribute("editor_filetab_file_id", &file_id.to_string()).unwrap();
        div.set_attribute("editor_filetab_file_path", &path).unwrap();

        let filename = path.split("/").last().unwrap();
        let path1 = path.clone();
        let open_callback = Closure::<dyn FnMut() -> ()>::new(move || { crate::state::open_file(file_id, path1.clone(), true); });
        div.add_event_listener_with_callback("click", open_callback.as_ref().unchecked_ref()).unwrap();
        open_callback.forget();

        let icon = document.create_element("div").unwrap();
        icon.class_list().toggle_with_force("editor_filetab_icon", true).unwrap();
        let icon_inner = document.create_element("i").unwrap();
        crate::filetree::set_filename_icon_classes(filename, &icon_inner.class_list());
        icon.append_child(&icon_inner).unwrap();
        div.append_child(&icon).unwrap();

        let name = document.create_element("div").unwrap();
        name.class_list().toggle_with_force("editor_filetab_name", true).unwrap();
        name.set_inner_html(filename);
        div.append_child(&name).unwrap();

        let close = document.create_element("div").unwrap();
        close.class_list().toggle_with_force("editor_filetab_close", true).unwrap();
        close.set_inner_html("×");
        div.append_child(&close).unwrap();
        let path1 = path.clone();
        let close_callback = Closure::<dyn FnMut(_) -> ()>::new(move |e : PointerEvent| {
            crate::state::close_file(file_id, Some(path1.clone()));
            e.stop_propagation();
        });
        close.add_event_listener_with_callback("click", close_callback.as_ref().unchecked_ref()).unwrap();
        close_callback.forget();

        filetabs.insert_before(&div, before.as_ref()).unwrap();
        div
    }).set_id("editor_filetab_selected");

    // File path
    set_filepath(&path);
}


pub fn close(file_id : u64) {
    let window   = web_sys::window().unwrap();
    let document = window.document().unwrap();

    // File tab
    let     filetabs = document.get_element_by_id("editor_filetabs").unwrap();
    let     children = filetabs.children();
    let mut found    = None;
    for i in 0..children.length() {
        let tab = children.get_with_index(i).unwrap();
        let other_file_id = tab.get_attribute("editor_filetab_file_id").unwrap().parse::<u64>().unwrap();
        if (file_id == other_file_id) {
            crate::code::destroy_monaco(file_id);
            if (tab.id() == "editor_filetab_selected") {
                tab.remove_attribute("id").unwrap();
                found = Some(i.saturating_sub(1));
            }
            filetabs.remove_child(&tab).unwrap();
            break;
        }
    }
    match (found.map(|i| filetabs.children().get_with_index(i))) {
        Some(Some(tab)) => {
            tab.set_id("editor_filetab_selected");
            let file_id   = tab.get_attribute("editor_filetab_file_id").unwrap().parse::<u64>().unwrap();
            let file_path = tab.get_attribute("editor_filetab_file_path").unwrap();
            set_filepath(&file_path);
            crate::filetree::open_file(file_id);
            if let Some(FilesEntry { is_open, .. }) = crate::state::FILES.read_files().get(&file_id) {
                match (is_open) {
                    Some(Some(FilesEntryContents::Text(_))) => { crate::code::open_monaco(file_id); },
                    Some(Some(FilesEntryContents::NonText)) => { crate::code::open_nontext(); },
                    Some(None) => { crate::code::open_load(); },
                    None => { crate::code::open_noopen(); }
                }
            }
        },
        Some(None) => {
            clear_filepath();
            crate::filetree::close_file(file_id);
            crate::code::open_noopen();
        },
        None => { }
    }
}


fn clear_filepath() {
    let window   = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let filepath = document.get_element_by_id("editor_filepath").unwrap();
    filepath.set_inner_html("");
}


fn set_filepath(path : &str) {
    let window   = web_sys::window().unwrap();
    let document = window.document().unwrap();

    clear_filepath();
    let filepath = document.get_element_by_id("editor_filepath").unwrap();
    for part in path.split("/") {
        let div = document.create_element("div").unwrap();
        div.set_inner_html(part);
        filepath.append_child(&div).unwrap();
    }
}


pub fn overwrite(file_id : u64, fsname : &str, contents : &FileContents) {
    let window   = web_sys::window().unwrap();
    let document = window.document().unwrap();

    // File tab
    let filetabs = document.get_element_by_id("editor_filetabs").unwrap();
    let children = filetabs.children();
    for i in 0..children.length() {
        let tab = children.get_with_index(i).unwrap();
        let other_file_id = tab.get_attribute("editor_filetab_file_id").unwrap().parse::<u64>().unwrap();
        if (file_id == other_file_id) {
            if (tab.id() == "editor_filetab_selected") {
                match (contents) {
                    FileContents::NonText => { crate::code::open_nontext(); },
                    FileContents::Text(text) => { crate::code::create_monaco(file_id, fsname, text, true) },
                }
            } else {
                match (contents) {
                    FileContents::NonText => { },
                    FileContents::Text(text) => { crate::code::create_monaco(file_id, fsname, text, false) },
                }
            }
            break;
        }
    }
}


pub fn currently_focused() -> Option<(u64, String)> {
    let window   = web_sys::window().unwrap();
    let document = window.document().unwrap();

    // File tab
    let filetabs = document.get_element_by_id("editor_filetabs").unwrap();
    let children = filetabs.children();
    for i in 0..children.length() {
        let tab = children.get_with_index(i).unwrap();
        if (tab.id() == "editor_filetab_selected") {
            return Some((
                tab.get_attribute("editor_filetab_file_id").unwrap().parse::<u64>().unwrap(),
                tab.get_attribute("editor_filetab_file_path").unwrap()
            ));
        }
    }
    None
}

pub fn list_all() -> Vec<(u64, String)> {
    let window   = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let mut ids = Vec::new();

    // File tab
    let filetabs = document.get_element_by_id("editor_filetabs").unwrap();
    let children = filetabs.children();
    for i in 0..children.length() {
        let tab = children.get_with_index(i).unwrap();
        ids.push((
            tab.get_attribute("editor_filetab_file_id").unwrap().parse::<u64>().unwrap(),
            tab.get_attribute("editor_filetab_file_path").unwrap()
        ));
    }
    ids
}
