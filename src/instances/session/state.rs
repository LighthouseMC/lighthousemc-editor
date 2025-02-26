use crate::peer::OutgoingPeerCommand;
use crate::instances::EditorInstance;
use super::{ EditorSession, EditorSessionStep };
use voxidian_editor_common::packet::s2c::*;
use voxidian_database::DBFSFileID;
use axecs::prelude::*;
use std::collections::BTreeMap;


pub struct EditorSessionState {
    file_shadows : BTreeMap<DBFSFileID, FileShadow>
}

pub struct FileShadow {
    step    : FileShadowStep,
    content : FileShadowContent
}

pub enum FileShadowStep {
    Opening,
    Open,
    Closing
}

pub enum FileShadowContent {
    Loading,
    NonText,
    Text(String)
}


impl EditorSessionState {

    pub(super) fn new() -> Self { Self {
        file_shadows : BTreeMap::new()
    } }

    pub(super) fn open_file(&mut self, file_id : DBFSFileID) {
        match (self.file_shadows.get_mut(&file_id)) {
            None => { self.file_shadows.insert(file_id, FileShadow {
                step    : FileShadowStep::Opening,
                content : FileShadowContent::Loading
            }); },
            Some(shadow) => { match (shadow.step) {
                FileShadowStep::Opening => { },
                FileShadowStep::Open    => { },
                FileShadowStep::Closing => { shadow.step = FileShadowStep::Opening; },
            } }
        }
    }

    pub(super) fn close_file(&mut self, file_id : DBFSFileID) {
        if let Some(shadow) = self.file_shadows.get_mut(&file_id) {
            shadow.step = FileShadowStep::Closing;
        }
    }

}



pub(crate) async fn update_state(
        instances : Entities<(&EditorInstance)>,
    mut sessions  : Entities<(&mut EditorSession)>
) {
    for session in &mut sessions {
        if let EditorSessionStep::Active { state, outgoing_commands_tx, .. } = &mut session.session_step {
            let Some(instance) = instances.iter().find(|instance| instance.plot_id == session.plot_id) else { continue; };

            // File shadows.
            {
                let mut remove = Vec::new();
                for (&file_id, shadow) in state.file_shadows.iter_mut() {
                    match (shadow.step) {
                        FileShadowStep::Opening => {
                            shadow.step = FileShadowStep::Open;
                            if let Some(file) = instance.state.files().get(&file_id) {
                                let _ = outgoing_commands_tx.send(OutgoingPeerCommand::Send(S2CPackets::OvewriteFile(OverwriteFileS2CPacket {
                                    file_id,
                                    contents : file.contents.clone()
                                })));
                            } else {
                                let _ = outgoing_commands_tx.send(OutgoingPeerCommand::Send(S2CPackets::CloseFile(CloseFileS2CPacket { file_id })));
                                remove.push(file_id);
                            }
                        },
                        FileShadowStep::Open    => { },
                        FileShadowStep::Closing => { remove.push(file_id); }
                    }
                }
                for file_id in remove {
                    state.file_shadows.remove(&file_id);
                }
            }

        }
    }
}
