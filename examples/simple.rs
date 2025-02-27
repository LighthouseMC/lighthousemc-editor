#![feature(mpmc_channel)]


use lighthousemc_editor::EditorPlugin;
use lighthousemc_editor::instances::EditorInstance;
use lighthousemc_editor::instances::session::EditorSession;
use lighthousemc_database::LighthouseDB;
use voxidian_logger::LOGS;
use axecs::prelude::*;
use std::sync::Arc;
use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Duration;
use std::thread;
use uuid::Uuid;


#[tokio::main]
async fn main() {
    thread::spawn(|| { while let Ok(log) = LOGS.copy_recv().recv() {
        println!("{}", log.level.stylise(&format!("\x1b[7m[ {} {} ]\x1b[27m {}", log.level.name(), log.time_fmt, log.message)));
    } });

    let db = Arc::new(LighthouseDB::connect(SocketAddr::from_str("127.0.0.1:5432").unwrap()).await.unwrap());

    let mut app = App::new();
    app.add_plugin(CycleSchedulerPlugin);
    app.add_plugin(CtrlCPlugin::default());
    app.add_plugin(EditorPlugin::new(
        "127.0.0.1:5123",
        "127.0.0.1:25565".into(),
        Arc::clone(&db)
    ).await.unwrap());

    app.add_systems(Startup, create_session_and_instance.pass(db));

    app.run().await;
}


async fn create_session_and_instance(
    In(database) : In<Arc<LighthouseDB>>,
    cmds         : Commands
) {
    let plot_id = 6;

    let instance = unsafe{ EditorInstance::create(plot_id, database) }.await.unwrap().unwrap();
    cmds.spawn(instance).await;

    let session = unsafe{ EditorSession::create_with(
        plot_id,
        Uuid::new_v4(),
        "Totobirb".into(),
        Duration::from_secs(60),
        "A".into()
    ) }.unwrap();
    voxidian_logger::pass!("http://127.0.0.1:5123/editor#DO-NOT-SHARE_{}", session.session_code());
    cmds.spawn(session).await;

    let session = unsafe{ EditorSession::create_with(
        plot_id,
        Uuid::new_v4(),
        "Other Person".into(),
        Duration::from_secs(60),
        "B".into()
    ) }.unwrap();
    voxidian_logger::pass!("http://127.0.0.1:5123/editor#DO-NOT-SHARE_{}", session.session_code());
    cmds.spawn(session).await;
}
