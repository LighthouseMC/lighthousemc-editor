use crate::peer::OutgoingPeerCommand;
use crate::instances::{ EditorInstance, EditorInstanceEvent };
use crate::util::Dirty;
use super::{ EditorSession, EditorSessionStep };
use voxidian_editor_common::packet::s2c::*;
use voxidian_editor_common::packet::c2s::SelectionRange;
use voxidian_database::DBFSFileID;
use axecs::prelude::*;
use std::collections::BTreeMap;


pub struct EditorSessionState {
    file_shadows : BTreeMap<DBFSFileID, FileShadow>,
    selections   : Dirty<Option<(DBFSFileID, Vec<SelectionRange>)>>
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
        file_shadows : BTreeMap::new(),
        selections   : Dirty::new_clean(None)
    } }

    pub fn selections(&self) -> &Option<(DBFSFileID, Vec<SelectionRange>)> {
        &*self.selections
    }


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

    pub(super) fn update_selections(&mut self, selections : Option<(DBFSFileID, Vec<SelectionRange>)>) {
        Dirty::set(&mut self.selections, selections);
    }

}



pub(crate) async fn update_state(
    mut instances : Entities<(&mut EditorInstance)>,
    mut sessions  : Entities<(&mut EditorSession)>
) {
    for session in &mut sessions {
        if let EditorSessionStep::Active { outgoing_commands_tx, state, .. } = &mut session.session_step {
            let Some(instance) = instances.iter_mut().find(|instance| instance.plot_id == session.plot_id) else { continue; };

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

            // Selections.
            if (Dirty::take_dirty(&mut state.selections)) {
                instance.events.push_back(EditorInstanceEvent::UpdateSelections { packet : SelectionsS2CPacket {
                    client_uuid : session.client_uuid,
                    client_name : session.client_name.clone().into(),
                    colour      : (session.client_uuid.as_u128() % 180) as u8,
                    selections  : (*state.selections).clone()
                } });
            }

        }
    }
}
