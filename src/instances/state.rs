use voxidian_editor_common::packet::s2c::InitialStateS2CPacket;
use voxidian_database::{ VoxidianDB, DBPlotID, DBError };


pub struct EditorState {
    plot_id         : DBPlotID,
    plot_owner_name : String
}

impl EditorState {

    pub async fn load(database : &VoxidianDB, plot_id : DBPlotID) -> Result<Option<Self>, DBError> {
        let Some(plot) = database.get_plot(plot_id).await? else { return Ok(None); };
        Ok(Some(Self {
            plot_id,
            plot_owner_name : database.get_player(plot.owning_player).await?.username
        }))
    }

    pub(crate) fn to_initial_state(&self) -> InitialStateS2CPacket {
        InitialStateS2CPacket {
            plot_id         : self.plot_id,
            plot_owner_name : (&self.plot_owner_name).into(),
            tree_entries    : (&[]).into()
        }
    }

}
