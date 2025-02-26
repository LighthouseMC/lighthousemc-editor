use crate::peer::OutgoingPeerCommand;
use voxidian_editor_common::packet::s2c::*;
use voxidian_database::{ VoxidianDB, DBPlotID, DBError };
use axecs::prelude::*;
use std::sync::Arc;
use std::collections::VecDeque;


pub mod session;
use session::{ EditorSession, EditorSessionStep };

mod state;
pub use state::EditorInstanceState;


#[derive(Component)]
pub struct EditorInstance {
    plot_id : DBPlotID,
    state   : EditorInstanceState,
    events  : VecDeque<EditorInstanceEvent>
}

impl EditorInstance {

    /// # Safety:
    /// The plot must not be managed by any other editor instance.
    /// The plot must be locked and unlocked properly, preventing management conflicts with other nodes.
    pub async unsafe fn create(plot_id : DBPlotID, database : Arc<VoxidianDB>) -> Result<Option<Self>, DBError> {
        Ok(Some(Self {
            plot_id,
            state   : { let Some(state) = EditorInstanceState::load(&database, plot_id).await? else { return Ok(None); }; state },
            events  : VecDeque::new()
        }))
    }


    pub fn plot_id(&self) -> DBPlotID {
        self.plot_id
    }

    pub fn state(&self) -> &EditorInstanceState {
        &self.state
    }

}


pub enum EditorInstanceEvent {
    UpdateSelections {
        packet : SelectionsS2CPacket<'static>
    }
}


pub(super) async fn read_instance_events(
    mut instances : Entities<(&mut EditorInstance)>,
        sessions  : Entities<(&EditorSession)>
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
            }

        } }
    }
}
