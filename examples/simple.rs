#![feature(mpmc_channel)]


use voxidian_editor::EditorPlugin;
use voxidian_logger::LOGS;
use voxidian_database::VoxidianDB;
use axecs::prelude::*;
use std::sync::Arc;
use std::net::SocketAddr;
use std::str::FromStr;
use std::thread;


#[tokio::main]
async fn main() {
    thread::spawn(|| { while let Ok(log) = LOGS.copy_recv().recv() {
        println!("{}", log.level.stylise(&format!("\x1b[7m[ {} {} ]\x1b[27m {}", log.level.name(), log.time_fmt, log.message)));
    } });

    let db = Arc::new(VoxidianDB::connect(SocketAddr::from_str("127.0.0.1:5432").unwrap()).await.unwrap());

    let mut app = App::new();
    app.add_plugin(CycleSchedulerPlugin);
    app.add_plugin(CtrlCPlugin::default());
    app.add_plugin(EditorPlugin::new(
        "127.0.0.1:5123".to_string(),
        "127.0.0.1:25565".to_string(),
        db
    ).await.unwrap());
    app.run().await;
}
