use voxidian_editor_common::packet::{ self, PrefixedPacketEncode };
use voxidian_editor_common::packet::s2c::*;
use std::time::{ Instant, Duration };
use axum::extract::ws;
use uuid::Uuid;


pub struct EditorSession {
    pub(super) session_code : String,
    pub(super) client_uuid  : Uuid,
    pub(super) client_name  : String,
    pub(super) mode         : EditorSessionMode
}
impl EditorSession {

    pub(super) fn activate(&mut self, socket : ws::WebSocket) {
        let EditorSessionMode::Pending(_) = self.mode else { panic!("`EditorSession::activate` called when non-pending") };
        self.mode = EditorSessionMode::Active(ActiveEditorSession::new(socket));
    }

    pub(super) async fn update(&mut self) -> Result<(), axum::Error> {
        if let EditorSessionMode::Active(active) = &mut self.mode {

            if (active.just_logged_in) {
                active.just_logged_in = false;
                active.send(LoginSuccessS2CPacket).await?;
                active.send(InitialStateS2CPacket {
                    plot_id         : 1,
                    plot_owner_name : "TODO".into(),
                    tree_entries    : (&[]).into(),
                }).await?;
            }

        }
        Ok(())
    }

}


pub enum EditorSessionMode {
    Pending(PendingEditorSession),
    Active(ActiveEditorSession),
    Closed
}


pub struct PendingEditorSession {
    pub(super) timeout : Instant
}
impl PendingEditorSession {

    pub(super) fn new(timeout_in : Duration) -> Self { Self {
        timeout : Instant::now() + timeout_in,
    } }

}


pub struct ActiveEditorSession {
    socket         : ws::WebSocket,
    just_logged_in : bool
}
impl ActiveEditorSession {

    fn new(socket : ws::WebSocket) -> Self { Self {
        socket,
        just_logged_in : true
    } }

    pub(super) async fn send(&mut self, packet : impl PrefixedPacketEncode) -> Result<(), axum::Error> {
        self.socket.send(ws::Message::Binary(packet::encode(packet).into())).await
    }

}
