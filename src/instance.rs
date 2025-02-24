use crate::session::*;
use voxidian_editor_common::packet::s2c::DisconnectS2CPacket;
use voxidian_logger::debug;
use voxidian_database::{ VoxidianDB, DBPlotID };
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;
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
            .or_insert_with(|| {
                debug!("Opened editor instance on plot {}.", plot_id);
                RwLock::new(EditorInstance::new(Arc::clone(&self.db), plot_id))
            })
            .write().await
    }

    pub async fn get_pending_session(&self, session_code : &str) -> Option<RwLockWriteGuard<EditorSession>> {
        for (_, instance) in &*self.instances.write().await {
            for session in &*instance.read().await.sessions {
                let session = session.write().await;
                if let EditorSessionMode::Pending(_) = &session.mode {
                    if (session.session_code == session_code) {
                        return Some(session);
                    }
                }
            }
        }
        None
    }

    pub async fn update(&self) {
        let instances = self.instances.read().await;
        for (_, instance) in &*instances { // TODO: Parallelise
            instance.write().await.update().await;
        }
        self.cleanup().await;
    }

    async fn cleanup(&self) {
        let mut instances = self.instances.write().await;
        // Clean up instances that have no sessions.
        let mut remove = Vec::new();
        for (plot_id, instance) in &*instances {
            if (instance.read().await.sessions.is_empty()) { // TODO: Parallelise
                remove.push(*plot_id);
            }
        }
        for plot_id in remove {
            debug!("Closed editor instance on plot {}.", plot_id);
            instances.remove(&plot_id);
        }
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

    pub async fn kill_and_create_session<const SESSION_CODE_LEN : usize>(
        &mut self,
        client_uuid : Uuid,
        client_name : String,
        timeout_in  : Duration
    ) -> (RwLockWriteGuard<EditorSession>, String) {
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
                    EditorSessionMode::Closed => { }
                }
                remove.push(i);
            }
        }
        for i in remove {
            debug!("Closed editor session for {} on plot {}.", client_uuid, self.state.plot_id);
            self.sessions.remove(i);
        }
        // Generate a session code.
        let mut bytes = [0; SESSION_CODE_LEN];
        rand::rand_priv_bytes(&mut bytes).unwrap();
        let session_code = bytes.map(|byte| Self::rand_byte_to_char(byte)).into_iter().collect::<String>();
        // Add the new session.
        self.sessions.push(RwLock::new(EditorSession {
            session_code : session_code.clone(),
            client_uuid,
            client_name,
            mode         : EditorSessionMode::Pending(PendingEditorSession::new(timeout_in)),
        }));
        debug!("Opened editor session for {} on plot {}.", client_uuid, self.state.plot_id);
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


    async fn update(&mut self) {
        for session in &self.sessions { // TODO: Parallelise
            if let Err(err) = session.write().await.update().await {
                voxidian_logger::warn!("{}", err);
                // TODO: Handle error and kick
            };
        }
        self.cleanup().await;
    }

    async fn cleanup(&mut self) {
        // TODO
    }

}


pub struct EditorState {
    db      : Arc<VoxidianDB>,
    plot_id : DBPlotID,
}
