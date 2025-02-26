use axecs::prelude::*;
use voxidian_database::{ VoxidianDB, DBPlotID, DBError };
use std::sync::Arc;


pub mod session;

pub mod state;
use state::EditorState;


#[derive(Component)]
pub struct EditorInstance {
    plot_id : DBPlotID,
    state   : EditorState
}

impl EditorInstance {

    /// # Safety:
    /// The plot must not be managed by any other editor instance.
    pub async unsafe fn create(plot_id : DBPlotID, database : Arc<VoxidianDB>) -> Result<Option<Self>, DBError> {
        Ok(Some(Self {
            plot_id,
            state   : { let Some(state) = EditorState::load(&database, plot_id).await? else { return Ok(None); }; state }
        }))
    }


    pub fn plot_id(&self) -> DBPlotID {
        self.plot_id
    }

    pub fn state(&self) -> &EditorState {
        &self.state
    }

}
