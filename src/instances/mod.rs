use crate::peer::OutgoingPeerCommand;
use lighthousemc_editor_common::packet::s2c::*;
use lighthousemc_editor_common::dmp;
use lighthousemc_database::{ LighthouseDB, DBPlotID, DBFSFileID, DBError };
use axecs::prelude::*;
use std::sync::Arc;
use std::collections::VecDeque;
use uuid::Uuid;


pub mod session;
use session::*;

mod state;
pub use state::*;


#[derive(Component)]
pub struct EditorInstance {
                plot_id : DBPlotID,
    pub(crate)  state   : EditorInstanceState,
                events  : VecDeque<EditorInstanceEvent>
}

impl EditorInstance {

    /// # Safety:
    /// The plot must not be managed by any other editor instance.
    /// The plot must be locked and unlocked properly, preventing management conflicts with other nodes.
    pub async unsafe fn create(plot_id : DBPlotID, database : Arc<LighthouseDB>) -> Result<Option<Self>, DBError> {
        Ok(Some(Self {
            plot_id,
            state   : { let Some(state) = EditorInstanceState::load(&database, plot_id).await? else { return Ok(None); }; state },
            events  : VecDeque::new()
        }))
    }


    pub fn plot_id(&self) -> DBPlotID { self.plot_id }

}


pub enum EditorInstanceEvent {

    UpdateSelections {
        packet : SelectionsS2CPacket<'static>
    },

    PatchFile {
        client_uuid : Uuid,
        file_id     : DBFSFileID,
        patches     : dmp::Patches<dmp::Efficient>
    }

}


pub(super) async fn read_instance_events(
    mut instances : Entities<(&mut EditorInstance)>,
    mut sessions  : Entities<(&mut EditorSession)>
) {
    for instance in &mut instances {
        while let Some(event) = instance.events.pop_front() { match (event) {

            EditorInstanceEvent::UpdateSelections { packet } => {
                for session in &sessions { if (session.plot_id() == instance.plot_id) {
                    if (session.client_uuid() != packet.client_uuid) {
                        if let EditorSessionStep::Active { outgoing_commands_tx, .. } = session.session_step() {
                            let _ = outgoing_commands_tx.send(OutgoingPeerCommand::Send(S2CPackets::Selections(packet.clone())));
                        }
                    }
                } }
            },

            EditorInstanceEvent::PatchFile { client_uuid, file_id, patches } => {
                if let Some(file) = instance.state.files_mut().get_mut(&file_id) {
                    if let FileContents::Text(central_text) = file.contents_mut() {
                        let dmp = dmp::DiffMatchPatch::new();
                        // Apply patch to server text on a best-effort basis.
                        if let Ok((new_central_text, _)) = dmp.patch_apply(&patches, &central_text) {
                            *central_text = new_central_text.into();
                            'iter_sessions : for session in &mut sessions { if (session.plot_id() == instance.plot_id) {
                                let session_client_uuid = session.client_uuid();
                                if let EditorSessionStep::Active { outgoing_commands_tx, state, .. } = session.session_step_mut() {
                                    if let Some(shadow) = state.file_shadows_mut().get_mut(&file_id) {
                                        if let FileShadowStep::Open = shadow.step() {
                                            if let FileShadowContent::Text { text : shadow_text, .. } = shadow.content_mut() {
                                                // Apply patches directly if this client is the source of the change.
                                                if (session_client_uuid == client_uuid) {
                                                    let dmp = dmp::DiffMatchPatch::new();
                                                    if let Ok((new_shadow_text, _)) = dmp.patch_apply(&patches, shadow_text) {
                                                        *shadow_text = new_shadow_text;
                                                    }
                                                }
                                                // Server text is diffed against the server shadow.
                                                if let Ok(diffs) = dmp.diff_main(shadow_text, &central_text) {
                                                    if let Ok(patches_to_client) = dmp.patch_make(dmp::PatchInput::new_diffs(&diffs)) {
                                                        *shadow_text = central_text.to_string();
                                                        let _ = outgoing_commands_tx.send(OutgoingPeerCommand::Send(S2CPackets::PatchFile(PatchFileS2CPacket {
                                                            file_id,
                                                            patches : patches_to_client
                                                        })));
                                                        continue 'iter_sessions;
                                                    }
                                                }
                                                *shadow_text = central_text.to_string();
                                            }
                                            // Failed to merge changes, resend file.
                                            let _ = outgoing_commands_tx.send(OutgoingPeerCommand::Send(S2CPackets::OvewriteFile(OverwriteFileS2CPacket {
                                                file_id,
                                                contents : FileContents::Text(central_text.to_string().into())
                                            })));
                                        }
                                    }
                                }
                            } }
                        }
                    }
                }
            }

        } }
    }
}
