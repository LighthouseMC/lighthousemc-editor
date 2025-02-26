use crate::peer::{ OutgoingPeerCommand, IncomingPeerEvent };
use voxidian_editor_common::packet::c2s::*;
use voxidian_database::DBPlotID;
use voxidian_logger::debug;
use axecs::prelude::*;
use std::time::{ Instant, Duration };
use tokio::sync::mpsc;
use openssl::rand::rand_priv_bytes;
use uuid::Uuid;


mod state;
pub use state::*;


#[derive(Component)]
pub struct EditorSession {
    plot_id      : DBPlotID,

    client_uuid  : Uuid,
    client_name  : String,

    session_code : String,
    session_step : EditorSessionStep,

    closed       : bool
}

pub(crate) enum EditorSessionStep {
    Pending {
        expires_at : Instant
    },
    Active {
        outgoing_commands_tx : mpsc::UnboundedSender<OutgoingPeerCommand>,
        incoming_events_rx   : mpsc::UnboundedReceiver<IncomingPeerEvent>,
        state                : EditorSessionState
    }
}


impl EditorSession {

    /// # SAFETY:
    /// Client uuid and the plot together must be unique.
    /// The plot must be managed by an editor instance.
    pub unsafe fn create<const SESSION_CODE_LEN : usize>(
        plot_id     : DBPlotID,
        client_uuid : Uuid,
        client_name : String,
        expires_in  : Duration
    ) -> Result<Self, ()> {
        let mut sesssion_code = [0; SESSION_CODE_LEN];
        let Ok(_) = rand_priv_bytes(&mut sesssion_code) else { return Err(()); };
        Ok(Self {
            plot_id,
            client_uuid,
            client_name,
            session_code : sesssion_code.map(|b| Self::rand_byte_to_char(b)).into_iter().collect::<String>(),
            session_step : EditorSessionStep::Pending {
                expires_at : Instant::now() + expires_in
            },
            closed       : false
        })
    }

    fn rand_byte_to_char(byte : u8) -> char {
        let byte = byte % 64;
        let ascii = if ((0..26).contains(&byte)) {
            65 + (byte - 0)
        } else if ((26..52).contains(&byte)) {
            97 + (byte - 26)
        } else if ((52..62).contains(&byte)) {
            48 + (byte - 52)
        } else if (byte == 62) {
            43
        } else {
            45
        };
        ascii as char
    }


    pub fn plot_id(&self) -> DBPlotID {
        self.plot_id
    }

    pub fn session_code(&self) -> &str {
        &self.session_code
    }

    pub(crate) fn session_step(&self) -> &EditorSessionStep {
        &self.session_step
    }
    pub(crate) fn activate(
        &mut self,
        outgoing_commands_tx : mpsc::UnboundedSender<OutgoingPeerCommand>,
        incoming_events_rx   : mpsc::UnboundedReceiver<IncomingPeerEvent>
    )  {
        let EditorSessionStep::Pending { .. } = self.session_step else {
            panic!("`EditorSession::activate` called on already activated `EditorSession`");
        };

        debug!("Opened editor session for {:?} on plot {}.", self.client_name, self.plot_id);
        self.session_step = EditorSessionStep::Active {
            outgoing_commands_tx,
            incoming_events_rx,
            state                : EditorSessionState::new()
        };
    }


    pub fn close(&mut self) {
        if (! self.closed) {
            self.closed = true;
            if let EditorSessionStep::Active { outgoing_commands_tx, .. } = &mut self.session_step {
                let _ = outgoing_commands_tx.send(OutgoingPeerCommand::Close);
            }
            debug!("Closed editor session of {:?} on plot {}.", self.client_name, self.plot_id);
        }
    }

}



pub(crate) async fn read_session_events(
        cmds     : Commands,
    mut sessions : Entities<(Entity, &mut EditorSession)>
) {
    for (entity, session) in &mut sessions {
        match (&mut session.session_step) {

            EditorSessionStep::Pending { expires_at } => {
                if (Instant::now() >= *expires_at) {
                    session.close();
                }
            },

            EditorSessionStep::Active { incoming_events_rx, state, .. } => {
                match (incoming_events_rx.try_recv()) {
                    Ok(event) => { match (event) {

                        IncomingPeerEvent::Recieve(packet) => { match (packet) {

                            C2SPackets::Keepalive(_) => { },

                            C2SPackets::OpenFile(OpenFileC2SPacket { file_id }) => {
                                state.open_file(file_id);
                            },

                            C2SPackets::CloseFile(CloseFileC2SPacket { file_id }) => {
                                state.close_file(file_id);
                            },

                            C2SPackets::PatchFile(_) => { },

                            C2SPackets::Selections(SelectionsC2SPacket { selections }) => {
                                state.update_selections(selections);
                            }

                        } },

                        IncomingPeerEvent::Close => { session.close(); }

                    } },
                    Err(mpsc::error::TryRecvError::Empty) => { },
                    Err(mpsc::error::TryRecvError::Disconnected) => { session.close(); }
                }
            }

        }
        if (session.closed) {
            cmds.despawn(entity).await;
        }
    }
}
