use crate::instances::EditorInstance;
use crate::instances::session::{ EditorSession, EditorSessionStep };
use voxidian_editor_common::packet::s2c::*;
use voxidian_editor_common::packet::c2s::*;
use axecs::prelude::*;
use tokio::sync::mpsc;
use tokio::task::yield_now;
use axum::extract::ws::WebSocket;


mod comms;


pub enum OutgoingPeerCommand {
    Send(S2CPackets<'static>),
    Close
}

pub enum IncomingPeerEvent {
    Recieve(C2SPackets),
    Close
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
    mut instances : Scoped<Entities<(&'static EditorInstance)>>,
    mut sessions  : Scoped<Entities<(&'static mut EditorSession)>>
) {
    // Find the relevant session.
    let mut result = None;
    {
        for (session) in &mut sessions.lock().await {
            if let EditorSessionStep::Pending { .. } = session.session_step() {
                if (&*handshake.session_code == session.session_code()) {
                    // Find the relevant instance.
                    for (instance) in &instances.lock().await {
                        if (instance.plot_id() == session.plot_id()) {

                            let (outgoing_commands_tx, outgoing_commands_rx) = mpsc::unbounded_channel();
                            let (incoming_events_tx, incoming_events_rx) = mpsc::unbounded_channel();
                            session.activate(outgoing_commands_tx, incoming_events_rx);
                            result = Some((
                                outgoing_commands_rx,
                                incoming_events_tx,
                                instance.state.to_initial_state_packet()
                            ));

                        }
                    }
                }
            }
        }
    };
    if let Some((outgoing_commands_rx, incoming_events_tx, initial_state_packet)) = result {
        run_editor_websocket(cmds, &mut socket, outgoing_commands_rx, &incoming_events_tx, initial_state_packet).await;
        let _ = incoming_events_tx.send(IncomingPeerEvent::Close);
        let _ = comms::send_packet(&mut socket, DisconnectS2CPacket { reason : "Connection interrupted".into() }).await;
    } else {
        let _ = comms::send_packet(&mut socket, DisconnectS2CPacket { reason : "Invalid session code. Has it expired?".into() }).await;
    }
}

async fn run_editor_websocket(
        cmds                 : Commands,
        socket               : &mut WebSocket,
    mut outgoing_commands_rx : mpsc::UnboundedReceiver<OutgoingPeerCommand>,
        incoming_events_tx   : &mpsc::UnboundedSender<IncomingPeerEvent>,
        initial_state_packet : InitialStateS2CPacket<'_>,
) {
    if let Err(_) = comms::send_packet(socket, initial_state_packet).await { return; };
    if let Err(_) = comms::send_packet(socket, LoginSuccessS2CPacket).await { return; };

    'main_loop : while (! cmds.is_exiting()) {

        match (comms::try_read_packet::<C2SPackets>(socket).await) {
            Ok(Some(packet)) => { if let Err(_) = incoming_events_tx.send(IncomingPeerEvent::Recieve(packet)) { break 'main_loop; } },
            Ok(None) => { }
            Err(_) => { break 'main_loop; },
        }

        'recv_outgoing : loop {
            match (outgoing_commands_rx.try_recv()) {
                Ok(command) => { match (command) {

                    OutgoingPeerCommand::Send(packet) => { match (comms::send_packet(socket, packet).await) {
                        Ok(_) => { },
                        Err(_) => { break 'main_loop; },
                    } },

                    OutgoingPeerCommand::Close => { break 'main_loop; }

                } },
                Err(mpsc::error::TryRecvError::Empty) => { break 'recv_outgoing; },
                Err(mpsc::error::TryRecvError::Disconnected) => { break 'main_loop; }
            }
        }

        yield_now().await;
    }
}
