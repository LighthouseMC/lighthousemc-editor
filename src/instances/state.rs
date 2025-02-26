use voxidian_editor_common::packet::s2c::{ InitialStateS2CPacket, FileTreeEntry };
use voxidian_database::{ VoxidianDB, DBPlotID, DBFSDirectoryID, DBFSDirectory, DBFSFileID, DBFSFile, DBError };
use std::collections::BTreeMap;


pub struct EditorState {
    plot_id         : DBPlotID,
    plot_owner_name : String,

    directories : BTreeMap<DBFSDirectoryID, DBFSDirectory>,
    files       : BTreeMap<DBFSFileID, DBFSFile>
}

impl EditorState {

    pub async fn load(database : &VoxidianDB, plot_id : DBPlotID) -> Result<Option<Self>, DBError> {
        let Some(plot) = database.get_plot(plot_id).await? else { return Ok(None); };
        Ok(Some(Self {
            plot_id,
            plot_owner_name : database.get_player(plot.owning_player).await?.username,
            directories     : {
                let mut map = BTreeMap::new();
                for directory in database.get_plot_directories(plot_id).await? {
                    map.insert(directory.id, directory);
                }
                map
            },
            files           : {
                let mut map = BTreeMap::new();
                for file in database.get_plot_files(plot_id).await? {
                    map.insert(file.id, file);
                }
                map
            }
        }))
    }

    pub(crate) fn to_initial_state(&self) -> InitialStateS2CPacket {
        InitialStateS2CPacket {
            plot_id         : self.plot_id,
            plot_owner_name : (&self.plot_owner_name).into(),
            tree_entries    : {
                let mut entries = Vec::with_capacity(self.directories.len() + self.files.len());
                for (_, directory) in &self.directories {
                    entries.push(FileTreeEntry {
                        entry_id   : directory.id,
                        is_dir     : true,
                        parent_dir : directory.parent_dir,
                        fsname     : directory.fsname.clone()
                    });
                }
                for (_, file) in &self.files {
                    entries.push(FileTreeEntry {
                        entry_id   : file.id,
                        is_dir     : false,
                        parent_dir : file.parent_dir,
                        fsname     : file.fsname.clone()
                    });
                }
                entries.into()
            }
        }
    }

}
