use voxidian_editor_common::packet::{ self, PrefixedPacketEncode };
use voxidian_editor_common::packet::s2c::DisconnectS2CPacket;
use voxidian_database::{ VoxidianDB, DBPlotID };
use std::collections::BTreeMap;
use std::sync::Arc;
use axecs::util::rwlock::{ RwLock, RwLockReadGuard, RwLockWriteGuard };
use axum::extract::ws;
use openssl::rand;
use uuid::Uuid;


pub struct EditorInstances {
    db        : Arc<VoxidianDB>,
    instances : RwLock<BTreeMap<DBPlotID, RwLock<EditorInstance>>>
}
impl EditorInstances {

    pub fn new(db : Arc<VoxidianDB>) -> Self { Self {
        db,
        instances : RwLock::new(BTreeMap::new())
    } }

    pub async fn get_or_create_instance(&self, plot_id : DBPlotID) -> RwLockWriteGuard<EditorInstance> {
        self.instances.write().await.entry(plot_id)
            .or_insert_with(|| RwLock::new(EditorInstance::new(Arc::clone(&self.db), plot_id)))
            .write().await
    }

    pub async fn get_pending_session(&self, session_code : &str) -> Option<RwLockWriteGuard<EditorSession>> {
        for (_, instance) in &*self.instances.read().await {
            for session in &*instance.read().await.sessions {
                let session = session.write().await;
                if let EditorSessionMode::Pending(pending) = &session.mode {
                    if (pending.session_code == session_code) {
                        return Some(session);
                    }
                }
            }
        }
        None
    }

}


pub struct EditorInstance {
    state    : EditorState,
    sessions : Vec<RwLock<EditorSession>>
}
impl EditorInstance {

    fn new(db : Arc<VoxidianDB>, plot_id : DBPlotID) -> Self { Self {
        state : EditorState {
            db,
            plot_id
        },
        sessions : Vec::new()
    } }

    pub async fn kill_and_create_session<const SESSION_CODE_LEN : usize>(&mut self, client_uuid : Uuid, client_name : String) -> (RwLockWriteGuard<EditorSession>, String) {
        // Kill previous sessions under the same client_uuid.
        let mut remove = Vec::new();
        for (i, session) in self.sessions.iter().enumerate().rev() {
            let session = session.read().await;
            if (session.client_uuid == client_uuid) {
                let mut session = RwLockReadGuard::upgrade(session).await;
                match (&mut session.mode) {
                    EditorSessionMode::Pending(_) => { },
                    EditorSessionMode::Active(active) => {
                        let _ = active.send(DisconnectS2CPacket { reason : "Logged in from another location".into()  }).await;
                    },
                }
                remove.push(i);
            }
        }
        for i in remove {
            self.sessions.remove(i);
        }
        // Generate a session code.
        let mut bytes = [0; SESSION_CODE_LEN];
        rand::rand_priv_bytes(&mut bytes).unwrap();
        let session_code = bytes.map(|byte| Self::rand_byte_to_char(byte)).into_iter().collect::<String>();
        // Add the new session.
        self.sessions.push(RwLock::new(EditorSession {
            client_uuid,
            client_name,
            mode        : EditorSessionMode::Pending(PendingEditorSession {
                session_code : session_code.clone()
            }),
        }));
        // Return
        (self.sessions.last().unwrap().write().await, session_code)
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

}


pub struct EditorState {
    db      : Arc<VoxidianDB>,
    plot_id : DBPlotID,
}


pub struct EditorSession {
    client_uuid : Uuid,
    client_name : String,
    mode        : EditorSessionMode
}
impl EditorSession {

    pub(super) fn activate(&mut self, socket : ws::WebSocket) {
        let EditorSessionMode::Pending(_) = self.mode else { panic!("`EditorSession::activate` called when non-pending") };
        self.mode = EditorSessionMode::Active(ActiveEditorSession::new(socket));
    }

}


pub enum EditorSessionMode {
    Pending(PendingEditorSession),
    Active(ActiveEditorSession)
}


pub struct PendingEditorSession {
    session_code : String
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

    async fn send(&mut self, packet : impl PrefixedPacketEncode) -> Result<(), axum::Error> {
        self.socket.send(ws::Message::Binary(packet::encode(packet).into())).await
    }

}
