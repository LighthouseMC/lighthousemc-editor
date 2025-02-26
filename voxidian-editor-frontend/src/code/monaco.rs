use crate::code::remote_cursors::REMOTE_SELECTIONS;
use crate::code::diffsync;
use std::cell::LazyCell;
use std::sync::{ RwLock, RwLockReadGuard, RwLockWriteGuard, Arc, Mutex };
use std::collections::HashMap;
use std::borrow::Cow;
use wasm_bindgen::prelude::*;
use web_sys::Element;
use js_sys::Array;
use serde::Serialize as Ser;
use serde::Deserialize as Deser;


pub(super) static EDITORS : EditorsContainer = EditorsContainer::new();
pub(super) struct EditorsContainer {
    files : LazyCell<RwLock<HashMap<u64, js::Editor>>>
}
impl EditorsContainer { const fn new() -> Self { Self {
    files : LazyCell::new(|| RwLock::new(HashMap::new()))
} } }
impl EditorsContainer {
    pub(super) fn read(&self) -> RwLockReadGuard<HashMap<u64, js::Editor>> {
        self.files.read().unwrap()
    }
    pub(super) fn write(&self) -> RwLockWriteGuard<HashMap<u64, js::Editor>> {
        self.files.write().unwrap()
    }
}
unsafe impl Sync for EditorsContainer { }


mod js { use super::*;

    #[wasm_bindgen]
    extern "C" {

        #[wasm_bindgen(js_namespace = require)]
        pub(super) fn config(config : &JsValue);

        pub(super) fn require(from : &JsValue, callback : &JsValue);

    }

    #[wasm_bindgen]
    extern "C" {

        /// https://microsoft.github.io/monaco-editor/docs.html#functions/editor.create.html
        #[wasm_bindgen(js_namespace = ["monaco", "editor"], js_name = "create")]
        pub(super) fn editor_create(on: &Element, config : &JsValue) -> Editor;

        /// https://microsoft.github.io/monaco-editor/docs.html#functions/editor.defineTheme.html
        #[wasm_bindgen(js_namespace = ["monaco", "editor"], js_name = "defineTheme")]
        pub(super) fn define_theme(name : &str, data : &JsValue) -> Editor;
    }

    #[wasm_bindgen]
    extern "C" {
        pub type Editor;

        /// https://microsoft.github.io/monaco-editor/docs.html#interfaces/editor.IStandaloneCodeEditor.html#getSelections.getSelections-1
        #[wasm_bindgen(method, js_name = "getSelections")]
        pub fn get_selections(this : &Editor) -> Vec<JsValue>;

        /// https://microsoft.github.io/monaco-editor/docs.html#interfaces/editor.IStandaloneCodeEditor.html#setSelections.setSelections-1
        #[wasm_bindgen(method, js_name = "setSelections")]
        pub fn set_selections(this : &Editor, selections : Vec<JsValue>);

        /// https://microsoft.github.io/monaco-editor/docs.html#interfaces/editor.IStandaloneCodeEditor.html#getModel.getModel-1
        #[wasm_bindgen(method, js_name = "getModel")]
        pub fn get_model(this : &Editor) -> EditorModel;

        /// https://microsoft.github.io/monaco-editor/docs.html#interfaces/editor.IStandaloneCodeEditor.html#onDidChangeModel
        #[wasm_bindgen(method, js_name = "onDidChangeModel")]
        pub fn on_did_change_model(this : &Editor, callback : &JsValue);

        /// https://microsoft.github.io/monaco-editor/docs.html#interfaces/editor.IStandaloneCodeEditor.html#onDidChangeCursorSelection
        #[wasm_bindgen(method, js_name = "onDidChangeCursorSelection")]
        pub fn on_did_change_cursor_selection(this : &Editor, callback : &JsValue);

        /// https://microsoft.github.io/monaco-editor/docs.html#interfaces/editor.IStandaloneCodeEditor.html#onDidChangeModelContent
        #[wasm_bindgen(method, js_name = "onDidChangeModelContent")]
        pub fn on_did_change_model_content(this : &Editor, callback : &JsValue);
    }

    #[wasm_bindgen]
    extern "C" {
        pub type EditorModel;

        /// https://microsoft.github.io/monaco-editor/docs.html#interfaces/editor.ITextModel.html#getValue.getValue-1
        #[wasm_bindgen(method, js_name = "getValue")]
        pub fn get_value(this : &EditorModel, eol_preference : u8) -> String;

        /// https://microsoft.github.io/monaco-editor/docs.html#interfaces/editor.ITextModel.html#setValue.setValue-1
        #[wasm_bindgen(method, js_name = "setValue")]
        pub fn set_value(this : &EditorModel, value : &str);

        /// https://microsoft.github.io/monaco-editor/typedoc/interfaces/editor.ITextModel.html#getOffsetAt.getOffsetAt-1
        #[wasm_bindgen(method, js_name = "getOffsetAt")]
        pub fn get_offset_at(this : &EditorModel, position : JsValue) -> usize;

        /// https://microsoft.github.io/monaco-editor/typedoc/interfaces/editor.ITextModel.html#getPositionAt.getPositionAt-1
        #[wasm_bindgen(method, js_name = "getPositionAt")]
        pub fn get_position_at(this : &EditorModel, offset : usize) -> JsValue;

        /// https://microsoft.github.io/monaco-editor/docs.html#interfaces/editor.ITextModel.html#deltaDecorations.deltaDecorations-1
        #[wasm_bindgen(method, js_name = "deltaDecorations")]
        pub fn delta_decorations(this : &EditorModel, old_decorations : Vec<String>, new_decorations : Vec<JsValue>) -> Vec<String>;

    }

}
pub use js::Editor;


#[derive(Ser)]
struct MonacoConfig<'l> {
    paths : MonacoConfigPaths<'l>
}
#[derive(Ser, Deser)]
struct MonacoConfigPaths<'l> {
    vs : Cow<'l, str>
}

#[derive(Ser, Deser)]
struct EditorTheme<'l> {
    base    : Cow<'l, str>,
    inherit : bool,
    #[serde(rename = "colors")]
    colours : EditorThemeColours<'l>,
    rules   : Cow<'l, [EditorThemeRule<'l>]>
}
#[derive(Ser, Deser)]
struct EditorThemeColours<'l> {
    #[serde(rename = "editor.lineHighlightBorder")]
    line_highlight_border           : Cow<'l, str>,
    #[serde(rename = "editor.selectionBackground")]
    selection_background            : Cow<'l, str>,
    #[serde(rename = "editor.findMatchBackground")]
    find_match_background           : Cow<'l, str>,
    #[serde(rename = "editor.findMatchHighlightBackground")]
    find_match_highlight_background : Cow<'l, str>,
    #[serde(rename = "focusBorder")]
    focus_border                    : Cow<'l, str>,
    #[serde(rename = "contrastBorder")]
    contrast_border                 : Cow<'l, str>
}
#[derive(Ser, Deser, Clone)]
struct EditorThemeRule<'l> {
    token : Cow<'l, str>
}

#[derive(Ser, Deser)]
struct EditorConfig<'l> {
    value                     : Cow<'l, str>,
    language                  : Cow<'l, str>,
    theme                     : Cow<'l, str>,
    #[serde(rename = "autoDetectHighContrast")]
    auto_detect_high_contrast : bool,
    #[serde(rename = "automaticLayout")]
    automatic_layout          : bool,
    #[serde(rename = "cursorBlinking")]
    cursor_blinking           : Cow<'l, str>,
    #[serde(rename = "fontFamily")]
    font_family               : Cow<'l, str>,
    #[serde(rename = "fontLigatures")]
    font_ligatures            : bool,
    #[serde(rename = "fontSize")]
    font_size                 : f32,
    #[serde(rename = "fontWeight")]
    font_weight               : Cow<'l, str>,
    minimap                   : EditorConfigMinimap<'l>,
    #[serde(rename = "renderFinalNewline")]
    render_final_newline      : Cow<'l, str>,
    #[serde(rename = "smoothScrolling")]
    smooth_scrolling          : bool
}
#[derive(Ser, Deser)]
struct EditorConfigMinimap<'l> {
    #[serde(rename = "showSlider")]
    show_slider : Cow<'l, str>,
    size        : Cow<'l, str>
}

#[derive(Ser, Deser, Debug)]
pub struct EditorSelection {
    #[serde(rename = "startLineNumber")]
    pub start_line   : usize,
    #[serde(rename = "startColumn")]
    pub start_column : usize,
    #[serde(rename = "endLineNumber")]
    pub end_line     : usize,
    #[serde(rename = "endColumn")]
    pub end_column   : usize
}
#[derive(Ser, Deser, Debug)]
pub struct EditorSetSelection {
    #[serde(rename = "selectionStartLineNumber")]
    pub start_line   : usize,
    #[serde(rename = "selectionStartColumn")]
    pub start_column : usize,
    #[serde(rename = "positionLineNumber")]
    pub end_line     : usize,
    #[serde(rename = "positionColumn")]
    pub end_column   : usize
}
#[derive(Ser, Deser, Debug)]
pub struct EditorPosition {
    #[serde(rename = "lineNumber")]
    pub line   : usize,
    pub column : usize
}

#[derive(Ser, Deser, Debug)]
pub struct EditorDecoration<'l> {
    pub options : EditorDecorationOptions<'l>,
    pub range   : EditorSelection
}
#[derive(Ser, Deser, Debug)]
pub struct EditorDecorationOptions<'l> {
    #[serde(rename = "className")]
    pub class_name    : Cow<'l, str>,
    #[serde(rename = "hoverMessage")]
    pub hover_message : EditorHoverMessage<'l>,
    #[serde(rename = "isWholeLine")]
    pub is_whole_line : bool,
    pub stickiness    : u8
}
#[derive(Ser, Deser, Debug)]
pub struct EditorHoverMessage<'l> {
    pub value : Cow<'l, str>
}

#[derive(Ser, Deser, Debug)]
pub struct SelectionChangedEvent {
    reason : u8
}
#[derive(Ser, Deser, Debug)]
pub struct ModelContentChangedEvent {
    #[serde(rename = "isFlush")]
    is_flush : bool
}


pub fn init_theme() {
    require(move || {

        js::define_theme("voxidian", &serde_wasm_bindgen::to_value(&EditorTheme {
            base    : "hc-black".into(),
            inherit : true,
            colours : EditorThemeColours {
                line_highlight_border           : "#007f00".into(),
                selection_background            : "#007f00".into(),
                find_match_background           : "#007f00".into(),
                find_match_highlight_background : "#007f00".into(),
                focus_border                    : "#00000000".into(),
                contrast_border                 : "#3f3f3f".into(),
            },
            rules   : (&[]).into()
        }).unwrap());

    });
}


pub fn create(file_id : u64, file_name : &str, initial_script : String, open : bool) { // TODO: Edit history undo/redo
    let initial_language = filename_to_language(file_name);
    require(move || {
        let window   = web_sys::window().unwrap();
        let document = window.document().unwrap();

        let container = document.create_element("div").unwrap();
        container.class_list().toggle_with_force("editor_code_container", true).unwrap();
        container.set_attribute("editor_code_file_id", &file_id.to_string()).unwrap();

        let code = document.create_element("div").unwrap();
        code.class_list().toggle_with_force("editor_code", true).unwrap();
        container.append_child(&code).unwrap();

        document.get_element_by_id("editor_right_main_container").unwrap().append_child(&container).unwrap();
        if (open) {
            container.class_list().toggle_with_force("editor_right_main_selected", true).unwrap();
        }

        let config = EditorConfig {
            value                     : (&initial_script).into(),
            language                  : initial_language.into(),
            theme                     : "voxidian".into(),
            auto_detect_high_contrast : false,
            automatic_layout          : true,
            cursor_blinking           : "smooth".into(),
            font_family               : "Fira Code".into(),
            font_ligatures            : true,
            font_size                 : 13.0,
            font_weight               : "350".into(),
            minimap                   : EditorConfigMinimap {
                show_slider : "always".into(),
                size        : "proportional".into()
            },
            render_final_newline      : "dimmed".into(),
            smooth_scrolling          : true
        };
        let editor = js::editor_create(&code, &serde_wasm_bindgen::to_value(&config).unwrap());

        let change_model_callback = Closure::<dyn FnMut(_) -> ()>::new(move |_ : js::Editor| {
            crate::code::remote_cursors::update();
        });
        editor.on_did_change_model(change_model_callback.as_ref().unchecked_ref());
        change_model_callback.forget();

        let change_selection_callback = Closure::<dyn FnMut(_) -> ()>::new(move |event : JsValue| {
            let event = serde_wasm_bindgen::from_value::<SelectionChangedEvent>(event).unwrap();
            if (event.reason != 1 && event.reason != 0) {
                if let Some(currently_focused) = currently_focused() && currently_focused == file_id {
                    super::selection_changed();
                }
            }
        });
        editor.on_did_change_cursor_selection(change_selection_callback.as_ref().unchecked_ref());
        change_selection_callback.forget();

        let old_content = Arc::new(Mutex::new(initial_script.clone()));
        let change_model_content_callback = Closure::<dyn FnMut(_) -> ()>::new(move |event : JsValue| {
            if let Some(editor) = EDITORS.read().get(&file_id) {
                let event = serde_wasm_bindgen::from_value::<ModelContentChangedEvent>(event).unwrap();
                let mut old_content = old_content.lock().unwrap();
                let     new_content = editor.get_model().get_value(1);
                if (! event.is_flush) {
                    // Shift remote cursors.
                    for (_, remote_selection) in &mut*REMOTE_SELECTIONS.write() {
                        if (remote_selection.file_id == file_id) {
                            for selection in &mut remote_selection.selections {
                                (selection.start, selection.end) = diffsync::shift_selection(&old_content, &new_content, selection.start, selection.end);
                            }
                        }
                    }
                    super::selection_changed();
                }
                *old_content = new_content;
            }
        });
        editor.on_did_change_model_content(change_model_content_callback.as_ref().unchecked_ref());
        change_model_content_callback.forget();

        crate::code::remote_cursors::update_known(file_id, &editor);

        EDITORS.write().insert(file_id, editor);
    });
}


pub fn open(file_id : u64) {
    let window   = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let containers = document.get_elements_by_class_name("editor_code_container");
    for i in 0..containers.length() {
        let container = containers.get_with_index(i).unwrap();
        if (container.get_attribute("editor_code_file_id").unwrap().parse::<u64>().unwrap() == file_id) {
            container.class_list().toggle_with_force("editor_right_main_selected", true).unwrap();
        }
    }
}


pub fn destroy(file_id : u64) {
    let window   = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let containers = document.get_elements_by_class_name("editor_code_container");
    for i in 0..containers.length() {
        let container = containers.get_with_index(i).unwrap();
        if (container.get_attribute("editor_code_file_id").unwrap().parse::<u64>().unwrap() == file_id) {
            document.get_element_by_id("editor_right_main_container").unwrap().remove_child(&container).unwrap();
            break;
        }
    }

    EDITORS.write().remove(&file_id);
}


pub fn currently_focused() -> Option<u64> {
    let window   = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let containers = document.get_elements_by_class_name("editor_code_container");
    for i in 0..containers.length() {
        let container = containers.get_with_index(i).unwrap();
        if (container.class_list().contains("editor_right_main_selected")) {
            return Some(container.get_attribute("editor_code_file_id").unwrap().parse::<u64>().unwrap());
        }
    }
    None
}


fn require<F : Fn() -> () + 'static>(f : F) {
    let config = MonacoConfig { paths : MonacoConfigPaths { vs : "https://unpkg.com/monaco-editor@latest/min/vs".into() } };
    js::config(&serde_wasm_bindgen::to_value(&config).unwrap());

    let from = Array::new();
    from.push(&JsValue::from_str("vs/editor/editor.main"));
    let callback = Closure::<dyn FnMut() -> ()>::new(move || f());
    js::require(from.unchecked_ref(), callback.as_ref().unchecked_ref());
    callback.forget();
}


pub fn filename_to_language(filename : &str) -> &'static str {
    if let Some(split) = filename.chars().position(|ch| ch == '.') {
        let (_, ext) = filename.split_at(split + 1);
        match (ext) {
            // Bash
            "sh" => "shell",
            // Rust
            "rs" => "rust",
            // Toml
            "toml" => "r",
            // Zig
            "zig" => "rust",

            _ => "plaintext"
        }
    } else { "plaintext" }
}
