//! https://neil.fraser.name/writing/sync/


use crate::state::FilesEntryContents;
use crate::code::monaco::{ self, EditorPosition, EditorSelection, EditorSetSelection };
use crate::code::remote_cursors;
use lighthousemc_editor_common::packet::c2s::PatchFileC2SPacket;
use lighthousemc_editor_common::dmp::{ DiffMatchPatch, Efficient, PatchInput, Patches };


pub fn send_patches_to_server() {
    let window   = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let containers = document.get_elements_by_class_name("editor_code_container");
    for i in 0..containers.length() {
        let container = containers.get_with_index(i).unwrap();
        let file_id = container.get_attribute("editor_code_file_id").unwrap().parse::<u64>().unwrap();

        let mut files = crate::state::FILES.write_files();
        let Some(Some(FilesEntryContents::Text(client_shadow))) = &mut files.get_mut(&file_id).unwrap().is_open else { continue };
        let client_text = monaco::EDITORS.read()[&file_id].get_model().get_value(1);

        if (client_shadow != &client_text) {
            let dmp = DiffMatchPatch::new();
            // Client Text is diffed against the Client Shadow.
            let diffs = dmp.diff_main::<Efficient>(&client_shadow, &client_text).unwrap();
            // This returns a list of edits which have been performed on Client Text.
            let patches = dmp.patch_make(PatchInput::new_diffs(&diffs)).unwrap();
            // Client Text is copied over to Client Shadow.
            *client_shadow = client_text;
            // The edits are sent to the Server.
            crate::ws::WS.send(PatchFileC2SPacket {
                file_id,
                patches
            });
        }
    }
}


pub fn apply_patches_from_server(file_id : u64, patches : Patches<Efficient>) {
    let window   = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let containers = document.get_elements_by_class_name("editor_code_container");
    for i in 0..containers.length() {
        let container = containers.get_with_index(i).unwrap();
        if (container.get_attribute("editor_code_file_id").unwrap().parse::<u64>().unwrap() == file_id) {
            let client_editor   = &monaco::EDITORS.read()[&file_id];
            let client_model    = client_editor.get_model();
            let old_client_text = client_model.get_value(1);
            let mut selections  = client_editor.get_selections().into_iter().map(|s| {
                let s = serde_wasm_bindgen::from_value::<EditorSelection>(s).unwrap();
                Selection {
                    start : client_model.get_offset_at(serde_wasm_bindgen::to_value(&EditorPosition { line : s.start_line, column : s.start_column }).unwrap()),
                    end   : client_model.get_offset_at(serde_wasm_bindgen::to_value(&EditorPosition { line : s.end_line,   column : s.end_column   }).unwrap())
                }
            }).collect::<Vec<_>>();

            let dmp = DiffMatchPatch::new();
            let mut intermediate_client_text = old_client_text;
            for patch in patches {
                let (new_client_text, _) = dmp.patch_apply(&vec![patch], &intermediate_client_text).unwrap();
                // Shift local cursor.
                for selection in &mut selections {
                    (selection.start, selection.end) = shift_selection(&intermediate_client_text, &new_client_text, selection.start, selection.end);
                }
                super::selection_changed();
                intermediate_client_text = new_client_text;
            }
            let new_client_text = intermediate_client_text.to_string();

            client_model.set_value(&new_client_text);

            // Update local cursor.
            client_editor.set_selections(selections.into_iter().map(|s| {
                let start = serde_wasm_bindgen::from_value::<EditorPosition>(client_model.get_position_at(s.start )).unwrap();
                let end   = serde_wasm_bindgen::from_value::<EditorPosition>(client_model.get_position_at(s.end   )).unwrap();
                serde_wasm_bindgen::to_value(&EditorSetSelection {
                    start_line   : start.line,
                    start_column : start.column,
                    end_line     : end.line,
                    end_column   : end.column
                }).unwrap()
            }).collect::<Vec<_>>());

            // Update remote cursors.
            /*for remote_selection in crate::code::remote_cursors::REMOTE_SELECTIONS.write().values_mut() {
                for selection in &mut remote_selection.selections {
                    (selection.start, selection.end) = shift_selection(&intermediate_client_text, &new_client_text, selection.start, selection.end);
                }
            }*/
            remote_cursors::update_known(file_id, client_editor);

            break;
        }
    }
}

pub(super) fn shift_selection(old_client_text : &str, new_client_text : &str, start : usize, end : usize) -> (usize, usize) {
    if (start > end) {
        let (end, start) = shift_selection_unchecked(old_client_text, new_client_text, end, start);
        (start, end)
    } else {
        shift_selection_unchecked(old_client_text, new_client_text, start, end)
    }
}
fn shift_selection_unchecked(old_client_text : &str, new_client_text : &str, start : usize, end : usize) -> (usize, usize) {
    let mut c = new_client_text.chars();
    let slice_start   = old_client_text.chars().position(|ch| c.next() != Some(ch)).unwrap_or(old_client_text.len());
    let mut c = new_client_text.chars().rev();
    let slice_end_old = old_client_text.chars().rev().position(|ch| c.next() != Some(ch)).map(|i| old_client_text.len() - i).unwrap_or(0);
    let mut c = old_client_text.chars().rev();
    let slice_end_new = new_client_text.chars().rev().position(|ch| c.next() != Some(ch)).map(|i| new_client_text.len() - i).unwrap_or(0);
    let a = start < slice_start;
    let b = start < slice_end_new + 1;
    let c = start < slice_end_old;
    let d = start + 1 < slice_end_new;
    let e = end < slice_start;
    let f = end < slice_end_old + 1;
    let g = end < slice_end_new + 1;
    let h = end < slice_end_old;
    let i = end < slice_end_new;
    let j = end + 1 < slice_end_old;
    let k = end + 1 < slice_end_new;
    let delta = (slice_end_old as isize - slice_start as isize) - (slice_end_new as isize - slice_start as isize);
    match ((a, b, c, d, e, f, g, h, i, j, k)) {

        (false, false, false, false, false, false, false, false, false, false, false)
        | (false, true, false, false, false, false, true, false, false, false, false)
        | (false, false, false, false, false, true, false, false, false, false, false)
        | (false, true, false, false, false, false, false, false, false, false, false)
        | (false, true, false, false, false, true, true, false, false, false, false)
        | (false, true, false, true, false, false, true, false, true, false, true)
        | (false, true, false, true, false, false, false, false, false, false, false)
        | (true, true, false, false, false, false, false, false, false, false, false)
        | (true, true, false, true, false, false, false, false, false, false, false)
        | (false, true, false, false, false, false, true, false, true, false, false)
        | (false, true, false, false, false, true, true, false, true, false, false)
        | (false, true, false, true, false, true, true, false, true, false, true)
        | (true, true, false, true, true, true, true, false, true, false, true)
        | (true, false, false, false, true, false, false, false, false, false, false)
        => (start.saturating_sub_signed(delta), end.saturating_sub_signed(delta)), // Change entirely before selection

        (false, false, true, false, false, false, false, false, false, false, false)
        | (false, true, true, false, false, false, false, false, false, false, false)
        => (slice_start, end.saturating_sub_signed(delta)), // Change crosses start of selection, but not end.

        (true, true, true, true, false, false, false, false, false, false, false)
        | (true, true, true, false, false, false, false, false, false, false, false)
        | (true, true, true, true, false, false, true, false, true, false, false)
        | (true, true, true, true, false, false, true, false, false, false, false)
        | (true, true, true, true, false, false, true, false, true, false, true)
        => (start, end.saturating_sub_signed(delta)), // Change entirely contained within selection

        (true, true, true, true, false, true, false, true, false, false, false)
        | (true, true, true, true, false, true, false, true, false, true, false)
        => (start, slice_end_new), // Change crosses end of selection, but not start

        (true, true, true, true, true, true, true, true, true, true, true)
        | (true, true, true, false, true, true, true, true, true, true, true)
        | (true, true, false, false, true, true, true, false, true, false, false)
        | (true, true, true, false, true,true, true, true, false, false, false)
        | (true, true, true, true, true,true, true, true, true, false, true)
        | (true, true, true, false, true, true, true, true, true, true, false)
        | (false, true, true, false, false, true, true, true, false, false, false)
        | (true, true, true, true, false, true, true, false, true, false, false)
        | (true, true, true, true, true, true, true, true, true, true, false)
        | (true, true, true, true, false, true, true, true, false, false, false)
        | (true, true, true, true, false, true, true, false, true, false, true)
        | (true, true, true, true, false, true, true, true, false, true, false)
        | (true, true, true, false, true, true, true, true, false, true, false)
        => (start, end), // Change entirely after selection

        (false, false, true, false, false, true, false, true, false, false, false)
        | (false, true, true, false, false, true, false, false, false, false, false)
        | (false, false, true, false, false, true, false, false, false, false, false)
        | (true, true, true, false, false, true, false, true, false, false, false)
        | (false, false, true, false, false, true, false, true, false, true, false)
        => (slice_end_new, slice_end_new), // Change entirely encapsulates selection

        _ => {
            crate::warn(&format!("MISSING ({}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}) MISSING", a, b, c, d, e, f, g, h, i, j, k));
            (start, end)
        }
    }
}

pub struct Selection {
    start : usize,
    end   : usize
}
