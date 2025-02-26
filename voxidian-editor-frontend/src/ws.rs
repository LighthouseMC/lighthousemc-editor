use crate::state::{ FilesEntry, FilesEntryContents };
use crate::code::remote_cursors::RemoteSelection;
use voxidian_editor_common::packet::{ PacketBuf, PacketEncode, PrefixedPacketEncode, PrefixedPacketDecode };
use voxidian_editor_common::packet::s2c::{ S2CPackets, FileContents };
use voxidian_editor_common::packet::c2s::*;
use voxidian_editor_common::dmp::DiffMatchPatch;
use std::cell::SyncUnsafeCell;
use std::ops::Deref;
use std::mem::MaybeUninit;
use std::sync::atomic::{ AtomicU64, Ordering };
use std::borrow::Cow;
use wasm_bindgen::prelude::*;
use web_sys::{ WebSocket, BinaryType, MessageEvent, ErrorEvent };
use js_sys::{ ArrayBuffer, Uint8Array };
use js_sys::Date;
use wasm_cookies as cookies;


pub static KEEPALIVE_INDEX : AtomicU64 = AtomicU64::new(0);


pub static WS : WebSocketContainer = WebSocketContainer::new();
pub struct WebSocketContainer {
    ws           : SyncUnsafeCell<MaybeUninit<WebSocket>>,
    session_code : SyncUnsafeCell<MaybeUninit<String>>
}
impl WebSocketContainer { const fn new() -> Self { Self {
    ws           : SyncUnsafeCell::new(MaybeUninit::uninit()),
    session_code : SyncUnsafeCell::new(MaybeUninit::uninit())
} } }
impl WebSocketContainer {
    pub fn session_code(&self) -> &str {
        unsafe{ (*self.session_code.get()).assume_init_ref() }
    }
    pub fn send<P : PacketEncode + PrefixedPacketEncode>(&self, packet : P) {
        self.send_with_u8_array(PacketBuf::of_encode_prefixed(packet).as_slice()).unwrap();
    }
}
impl Deref for WebSocketContainer {
    type Target = WebSocket;
    fn deref(&self) -> &Self::Target { unsafe{ (*self.ws.get()).assume_init_ref() } }
}
unsafe impl Sync for WebSocketContainer { }


pub(super) fn start() {
    let     window       = web_sys::window().unwrap();
    let     location     = window.location();
    let mut session_code = {
        let s = location.hash().unwrap();
        let s = s.strip_prefix("#").unwrap_or(&s);
        s.strip_prefix("DO-NOT-SHARE_").unwrap_or(&s).to_string()
    };
    if (session_code.is_empty()) {
        if let Some(Ok((sc))) = cookies::get("voxidian-editor-session") {
            session_code = sc;
        } else {
            crate::cover::open_cover_error(&format!("No session code"));
            return;
        }
    }
    let cookie_expires = Date::new_0();
    let cookie_expires_hours = 12.0;
    cookie_expires.set_time(cookie_expires.get_time() + (cookie_expires_hours*3600000.0));
    cookies::set("voxidian-editor-session", &session_code, &cookies::CookieOptions {
        path      : Some("/editor"),
        domain    : None,
        expires   : Some(Cow::Owned(cookie_expires.to_utc_string().into())),
        secure    : true,
        same_site : cookies::SameSite::Strict,
    });
    let protocol = match (location.protocol().unwrap().as_str()) {
        "http:" => "ws:",
        "https:" => "wss:",
        _ => panic!()
    };
    let hostname = location.hostname().unwrap();
    let port     = location.port().unwrap();
    let path     = location.pathname().unwrap();
    let path     = path.trim_end_matches('/');
    let ws_host  = format!("{protocol}//{hostname}:{port}{path}/ws");
    let ws       = WebSocket::new_with_str(&ws_host, "voxidian-editor").unwrap();
    ws.set_binary_type(BinaryType::Arraybuffer);

    let onerror_callback = Closure::<dyn FnMut(_) -> ()>::new(|e| on_ws_error(e));
    ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
    onerror_callback.forget();

    let onopen_callback = Closure::<dyn FnMut() -> ()>::new(|| on_ws_open());
    ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
    onopen_callback.forget();

    let onclose_callback = Closure::<dyn FnMut() -> ()>::new(|| on_ws_close());
    ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
    onclose_callback.forget();

    let onmessage_callback = Closure::<dyn FnMut(_) -> ()>::new(|e| on_ws_message(e));
    ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    onmessage_callback.forget();

    unsafe{ (*WS.ws           .get()).write(ws           ); }
    unsafe{ (*WS.session_code .get()).write(session_code ); }
}


fn on_ws_error(e : ErrorEvent) {
    crate::errorjs2("Error in connection:\n".into(), (&e).into());
    crate::cover::open_cover_error(&format!("<b>Error in connection</b>:<br />{:?}", e));
    let _ = WS.close();
}


fn on_ws_close() {
    crate::cover::open_cover_error(&format!("<b>Server disconnected</b><br />Something went wrong."));
    let _ = WS.close();
}


fn on_ws_open() {
    WS.send(HandshakeC2SPacket {
        session_code : WS.session_code().into(),
    });

    let timeout_callback = Closure::<dyn FnMut() -> ()>::new(move || {
        crate::code::diffsync::send_patches_to_server();
    });
    crate::set_interval(timeout_callback.as_ref().unchecked_ref(), 250);
    timeout_callback.forget();
}


fn on_ws_message(e : MessageEvent) {
    let     data   = Uint8Array::new(&e.data().dyn_into::<ArrayBuffer>().unwrap()).to_vec();
    let mut buf    = PacketBuf::from(data);
    let     packet = S2CPackets::decode_prefixed(&mut buf).unwrap();
    match (packet) {


        S2CPackets::Disconnect(disconnect) => {
            crate::cover::open_cover_error(&format!("<b>Server disconnected</b>:<br />{}", disconnect.reason));
            let _ = WS.close();
        },


        S2CPackets::Keepalive(_) => {
            let index = KEEPALIVE_INDEX.fetch_add(1, Ordering::SeqCst);
            WS.send(KeepaliveC2SPacket {
                index
            });
            let callback = Closure::<dyn FnMut() -> ()>::new(move || {
                if (KEEPALIVE_INDEX.load(Ordering::SeqCst) == index.wrapping_add(1)) {
                    crate::cover::open_cover_error(&format!("<b>Server disconnected</b><br />Timed out"));
                    let _ = WS.close();
                }
            });
            web_sys::window().unwrap().set_timeout_with_callback_and_timeout_and_arguments_0(callback.as_ref().unchecked_ref(), 5000).unwrap();
            callback.forget();
        },


        S2CPackets::LoginSuccess(_) => {
            crate::cover::close_cover_loader();
        },


        S2CPackets::InitialState(initial_state) => {
            let window   = web_sys::window().unwrap();
            let document = window.document().unwrap();
            // Plot id
            let plot_id  = initial_state.plot_id.to_string();
            let plot_ids = document.get_elements_by_class_name("template_plot_id");
            for i in 0..plot_ids.length() {
                plot_ids.get_with_index(i).unwrap().set_inner_html(&plot_id);
            }
            // Plot owner name
            let plot_owner_names = document.get_elements_by_class_name("template_plot_owner_name");
            for i in 0..plot_owner_names.length() {
                plot_owner_names.get_with_index(i).unwrap().set_inner_html(&initial_state.plot_owner_name);
            }
            // File tree
            crate::filetree::clear();
            for entry in &*initial_state.tree_entries {
                crate::state::add_tree_entry(entry.clone());
            }
            crate::filetree::sort();
        },


        S2CPackets::OvewriteFile(overwrite_file) => {
            if let Some(FilesEntry { is_open, fsname, .. }) = crate::state::FILES.write_files().get_mut(&overwrite_file.file_id) {
                crate::filetabs::overwrite(overwrite_file.file_id, fsname, &overwrite_file.contents);
                *is_open = Some(Some(match (overwrite_file.contents) {
                    FileContents::NonText    => FilesEntryContents::NonText,
                    FileContents::Text(text) => FilesEntryContents::Text(text.into_owned())
                }));
            }
        },


        S2CPackets::PatchFile(patch_file) => {
            if let Some(FilesEntry { is_open : Some(Some(FilesEntryContents::Text(old_client_shadow))), .. }) = crate::state::FILES.write_files().get_mut(&patch_file.file_id) {
                let dmp = DiffMatchPatch::new();
                let (new_client_shadow, _) = dmp.patch_apply(&patch_file.patches, &old_client_shadow).unwrap();
                *old_client_shadow = new_client_shadow;
                crate::code::diffsync::apply_patches_from_server(patch_file.file_id, patch_file.patches);
            }
        },


        S2CPackets::Selections(selections_event) => {
            if let Some((file_id, selections)) = selections_event.selections {
                crate::code::remote_cursors::REMOTE_SELECTIONS.write().insert(selections_event.client_uuid, RemoteSelection {
                    client_name : selections_event.client_name.into_owned(),
                    colour      : selections_event.colour,
                    file_id,
                    selections,
                });
            } else {
                crate::code::remote_cursors::REMOTE_SELECTIONS.write().remove(&selections_event.client_uuid);
            }
            crate::code::remote_cursors::update();
        },


        S2CPackets::CloseFile(close_file) => {
            crate::state::close_file(close_file.file_id, None);
        }


    }
}
