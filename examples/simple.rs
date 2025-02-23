use voxidian_editor::{ EditorServer, EditorInstances };
use voxidian_database::VoxidianDB;
use std::sync::Arc;
use std::net::SocketAddr;
use std::str::FromStr;
use uuid::Uuid;


#[tokio::main]
async fn main() {

    let db = Arc::new(VoxidianDB::connect(SocketAddr::from_str("127.0.0.1:5432").unwrap()).await.unwrap());

    let instances = Arc::new(EditorInstances::new(db));
    {
        let mut instance = instances.get_or_create_instance(1).await;
        let (_, code) = instance.kill_and_create_session::<192>(Uuid::new_v4(), "Totobirb".to_string()).await;
        println!("http://127.0.0.1:5123/editor#{}", code);
    }

    let server = EditorServer {
        display_game_address : "127.0.0.1:25565".to_string(),
        instances
    };
    server.run("127.0.0.1:5123").await.unwrap();
}
