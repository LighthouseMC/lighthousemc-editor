use crate::instances::EditorInstance;
use crate::instances::session::{ EditorSession, EditorSessionStep };
use voxidian_editor_common::packet::s2c::*;
use voxidian_editor_common::packet::c2s::*;
use axecs::prelude::*;
use tokio::sync::mpsc;
use axum::extract::ws::WebSocket;


pub(crate) mod comms;


pub enum OutgoingPeerCommand<'l> {
    Send(S2CPackets<'l>),
    Close
}

pub enum IncomingPeerEvent {
    Recieve(C2SPackets<'static>)
}


#[derive(Component)]
pub struct EditorPeer<'l> {
    outgoing_commands_tx : mpsc::UnboundedSender<OutgoingPeerCommand<'l>>,
    incoming_events_rx   : mpsc::UnboundedReceiver<IncomingPeerEvent>
}


pub(super) async fn handle_editor_websocket(cmds : Commands, mut socket : WebSocket) {
    let Ok(handshake) = comms::read_packet::<HandshakeC2SPacket>(&mut socket).await else { return; };
    let mut socket = Some(socket);
    cmds.run_system_mut(async move |cmds, instances, sessions| {
        try_login_editor_websocket(cmds, socket.take().unwrap(), &handshake, instances, sessions).await;
    }).await;
}

async fn try_login_editor_websocket(
        cmds      : Commands,
    mut socket    : WebSocket,
        handshake : &HandshakeC2SPacket<'_>,
        instances : Entities<(&EditorInstance)>,
    mut sessions  : Entities<(&mut EditorSession)>
) {
    // Find the relevant session.
    for (session) in &mut sessions {
        if let EditorSessionStep::Pending = session.session_step() {
            if (&*handshake.session_code == session.session_code()) {
                // Find the relevant instance.
                for instance in &instances {
                    if (instance.plot_id() == session.plot_id()) {
                        session.activate(socket, instance).await;
                        return;
                    }
                }
                session.close();
                break;
            }
        }
    }
    let _ = comms::send_packet(&mut socket, DisconnectS2CPacket { reason : "Invalid session code. Has it expired?".into() }).await;
}
