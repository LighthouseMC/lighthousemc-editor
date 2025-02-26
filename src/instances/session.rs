use crate::peer::comms;
use super::EditorInstance;
use voxidian_editor_common::packet::s2c::*;
use voxidian_database::DBPlotID;
use axecs::prelude::*;
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
    Pending,
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
        client_name : String
    ) -> Result<Self, ()> {
        let mut sesssion_code = [0; SESSION_CODE_LEN];
        let Ok(_) = rand_priv_bytes(&mut sesssion_code) else { return Err(()); };
        Ok(Self {
            plot_id,
            client_uuid,
            client_name,
            session_code : sesssion_code.map(|b| Self::rand_byte_to_char(b)).into_iter().collect::<String>(),
            session_step : EditorSessionStep::Pending,
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
        let EditorSessionStep::Pending = self.session_step else { panic!("`EditorSession::activate` called on already activated `EditorSession`") };

        if let Err(_) = comms::send_packet(&mut socket, instance.state().to_initial_state()).await { self.close(); }

        if let Err(_) = comms::send_packet(&mut socket, LoginSuccessS2CPacket).await { self.close(); }

        self.session_step = EditorSessionStep::Active {
            socket
        };
    }


    pub fn close(&mut self) {
        self.closed = true;
    }

}
