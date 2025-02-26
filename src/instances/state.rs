use voxidian_editor_common::packet::s2c::{ InitialStateS2CPacket, FileTreeEntry, FileContents };
use voxidian_database::{ VoxidianDB, DBPlotID, DBFSDirectoryID, DBFSDirectory, DBFSFileID, DBError };
use std::collections::BTreeMap;


pub struct EditorInstanceState {
    plot_id         : DBPlotID,
    plot_owner_name : String,

    directories : BTreeMap<DBFSDirectoryID, DBFSDirectory>,
    files       : BTreeMap<DBFSFileID, StateFile>
}

impl EditorInstanceState {

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
                    map.insert(file.id, StateFile {
                        parent_dir : file.parent_dir,
                        fsname     : file.fsname,
                        contents   : String::from_utf8(file.blob).map_or(FileContents::NonText, |text| FileContents::Text(text.into()))
                    });
                }
                map
            }
        }))
    }

    pub(crate) fn to_initial_state_packet(&self) -> InitialStateS2CPacket<'static> {
        InitialStateS2CPacket {
            plot_id         : self.plot_id,
            plot_owner_name : self.plot_owner_name.clone().into(),
            tree_entries    : {
                let mut entries = Vec::with_capacity(self.directories.len() + self.files.len());
                for (_, directory) in &self.directories {
                    entries.push(FileTreeEntry {
                        entry_id   : directory.id,
                        is_dir     : true,
                        parent_dir : directory.parent_dir,
                        fsname     : directory.fsname.clone().into()
                    });
                }
                for (file_id, file) in &self.files {
                    entries.push(FileTreeEntry {
                        entry_id   : *file_id,
                        is_dir     : false,
                        parent_dir : file.parent_dir,
                        fsname     : file.fsname.clone().into()
                    });
                }
                entries.into()
            }
        }
    }


    pub(crate) fn files(&self) -> &BTreeMap<DBFSFileID, StateFile> {
        &self.files
    }
    pub(crate) fn files_mut(&mut self) -> &mut BTreeMap<DBFSFileID, StateFile> {
        &mut self.files
    }

}


pub struct StateFile {
    parent_dir : Option<DBFSDirectoryID>,
    fsname     : String,
    contents   : FileContents<'static>
}

impl StateFile {

    pub fn contents(&self) -> &FileContents<'static> {
        &self.contents
    }
    pub fn contents_mut(&mut self) -> &mut FileContents<'static> {
        &mut self.contents
    }

}
