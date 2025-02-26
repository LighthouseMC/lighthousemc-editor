use crate::peer::comms;
use super::EditorInstance;
use voxidian_editor_common::packet::s2c::*;
use voxidian_editor_common::packet::c2s::*;
use voxidian_database::DBPlotID;
use voxidian_logger::debug;
use axecs::prelude::*;
use std::time::{ Instant, Duration };
use axum::extract::ws::WebSocket;
use openssl::rand::rand_priv_bytes;
use uuid::Uuid;


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
        socket : WebSocket
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
    pub(crate) async fn activate(&mut self, mut socket : WebSocket, instance : &EditorInstance)  {
        let EditorSessionStep::Pending { .. } = self.session_step else {
            panic!("`EditorSession::activate` called on already activated `EditorSession`");
        };

        if let Err(_) = comms::send_packet(&mut socket, instance.state().to_initial_state()).await { self.close(); }

        if let Err(_) = comms::send_packet(&mut socket, LoginSuccessS2CPacket).await { self.close(); }

        debug!("Opened editor session for {:?} on plot {}.", self.client_name, self.plot_id);
        self.session_step = EditorSessionStep::Active {
            socket
        };
    }


    pub fn close(&mut self) {
        if (! self.closed) {
            debug!("Closed editor session of {:?} on plot {}.", self.client_name, self.plot_id);
            self.closed = true;
        }
    }

}



pub(crate) async fn read_session_packets(
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

            EditorSessionStep::Active { socket } => {
                match (comms::try_read_packet::<C2SPackets>(socket).await) {
                    Ok(Some(packet)) => { match (packet) {

                        C2SPackets::Handshake(_) => { },

                        C2SPackets::Keepalive(keepalive_c2_spacket) => todo!(),

                        C2SPackets::OpenFile(OpenFileC2SPacket { file_id }) => todo!(),

                        C2SPackets::CloseFile(close_file_c2_spacket) => todo!(),

                        C2SPackets::PatchFile(patch_file_c2_spacket) => todo!(),

                        C2SPackets::Selections(selections_c2_spacket) => todo!()

                    } },
                    Ok(None) => { },
                    Err(_) => { session.close(); }
                }
            }

        }
        if (session.closed) {
            cmds.despawn(entity).await;
        }
    }
}
