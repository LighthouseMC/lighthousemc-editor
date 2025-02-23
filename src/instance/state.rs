use voxidian_database::{ VoxidianDB, DBPlotID, DBFSDirectoryID, DBFSDirectory, DBFSFileID, DBFSFile, DBError };
use std::time::{ Instant, Duration };
use std::sync::Arc;
use std::collections::HashMap;


pub(crate) struct EditorInstanceState {
    db         : Arc<VoxidianDB>,
    plot       : DBPlotID,
    owner_name : String,

    directories : HashMap<DBFSDirectoryID, DBFSDirectory>,
    files       : HashMap<DBFSFileID, DBFSFile>,

    edited     : bool,
    last_saved : Instant
}
impl EditorInstanceState {

    pub fn owner_name(&self) -> &str {
        &self.owner_name
    }

    pub fn directories_ref(&self) -> &HashMap<DBFSDirectoryID, DBFSDirectory> {
        &self.directories
    }

    pub fn files_ref(&self) -> &HashMap<DBFSFileID, DBFSFile> {
        &self.files
    }

}
impl EditorInstanceState {

    pub async fn open(db : Arc<VoxidianDB>, plot : DBPlotID) -> Result<Self, DBError> {
        let db_plot   = db.get_plot(plot).await?.ok_or(DBError::RowNotFound)?;
        let db_player = db.get_player(db_plot.owning_player).await?;
        let mut directories = HashMap::new();
        for directory in db.get_plot_directories(plot).await? {
            directories.insert(directory.id, directory);
        }
        let mut files = HashMap::new();
        for file in db.get_plot_files(plot).await? {
            files.insert(file.id, file);
        }
        Ok(Self {
            db,
            plot,
            owner_name : db_player.username,

            directories,
            files,

            edited     : false,
            last_saved : Instant::now()
        })
    }

    pub async fn save(&mut self) {
        if (! self.edited) {
            self.last_saved = Instant::now();
            return;
        }
        self.edited = false;
        todo!();
    }

    pub async fn autosave(&mut self) {
        if (Instant::now() >= (self.last_saved + Duration::from_secs(5))) {
            if (! self.edited) {
                self.last_saved = Instant::now();
                return;
            }
            self.edited = false;
            self.save();
        }
    }

}
