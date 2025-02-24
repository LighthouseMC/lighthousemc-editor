#![feature(mpmc_channel)]


use voxidian_editor::{ EditorServer, EditorHandle };
use voxidian_logger::LOGS;
use voxidian_database::VoxidianDB;
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

    let mut handle = EditorHandle::new();
    /*{
        let mut instance = instances.get_or_create_instance(1).await;
        let (_, code) = instance.kill_and_create_session::<192>(
            Uuid::new_v4(), "Totobirb".to_string(),
            Duration::from_secs(60)
        ).await;
        pass!("http://127.0.0.1:5123/editor#{}", code);
    }*/

    let server = EditorServer::new(
        "127.0.0.1:25565".to_string(),
        &mut handle,
        db
    );

    server.run("127.0.0.1:5123").await.unwrap();
}
