use axecs::prelude::*;
use voxidian_database::{ VoxidianDB, DBPlotID, DBError };
use std::sync::Arc;


pub mod session;

mod state;
pub use state::EditorInstanceState;


#[derive(Component)]
pub struct EditorInstance {
    plot_id : DBPlotID,
    state   : EditorInstanceState
}

impl EditorInstance {

    /// # Safety:
    /// The plot must not be managed by any other editor instance.
    /// The plot must be locked and unlocked properly, preventing management conflicts with other nodes.
    pub async unsafe fn create(plot_id : DBPlotID, database : Arc<VoxidianDB>) -> Result<Option<Self>, DBError> {
        Ok(Some(Self {
            plot_id,
            state   : { let Some(state) = EditorInstanceState::load(&database, plot_id).await? else { return Ok(None); }; state }
        }))
    }


    pub fn plot_id(&self) -> DBPlotID {
        self.plot_id
    }

    pub fn state(&self) -> &EditorInstanceState {
        &self.state
    }

}
