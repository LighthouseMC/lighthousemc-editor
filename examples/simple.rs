#![feature(mpmc_channel)]


use voxidian_editor::EditorPlugin;
use voxidian_editor::instances::EditorInstance;
use voxidian_editor::instances::session::EditorSession;
use voxidian_logger::LOGS;
use voxidian_database::VoxidianDB;
use axecs::prelude::*;
use std::sync::Arc;
use std::net::SocketAddr;
use std::str::FromStr;
use std::thread;
use uuid::Uuid;


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
        Arc::clone(&db)
    ).await.unwrap());

    app.add_systems(Startup, create_session_and_instance.pass(db));

    app.run().await;
}


async fn create_session_and_instance(
    In(database) : In<Arc<VoxidianDB>>,
    cmds         : Commands
) {
    let plot_id = 6;

    let instance = unsafe{ EditorInstance::create(plot_id, database) }.await.unwrap().unwrap();
    cmds.spawn(instance).await;

    let session = unsafe{ EditorSession::create::<192>(
        plot_id,
        Uuid::new_v4(),
        "Totobirb".into()
    ) }.unwrap();
    voxidian_logger::pass!("http://127.0.0.1:5123/editor#{}", session.session_code());
    cmds.spawn(session).await;
}
