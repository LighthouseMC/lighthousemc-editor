#![feature(
    decl_macro
)]


use voxidian_editor_common::packet;
use voxidian_database::VoxidianDB;
use std::io;
use std::time::Duration;
use std::sync::Arc;
use tokio::sync::{ mpsc, Mutex };
use tokio::net::{ TcpListener, ToSocketAddrs };
use tokio::time::timeout;
use axum::{ self, Router };
use axum::http::StatusCode;
use axum::http::{ HeaderValue, header::CONTENT_TYPE };
use axum::routing;
use axum::response::{ IntoResponse, Html };
use axum::extract::State;
mod mime {
    pub const TEXT : &'static str = "text/plain";
    pub const PNG  : &'static str = "image/png";
    pub const JS   : &'static str = "application/javascript";
    pub const WASM : &'static str = "application/wasm";
}
use axum::extract::ws;

use const_format::str_replace;
macro str_replace_multiple {

    ( $original:expr $( , [ $(,)? ] )? $(,)? ) => { $original },

    ( $original:expr , [ ( $aa:expr , $ab:expr $(,)? ) $( , ( $ba:expr , $bb:expr $(,)? ) )* $(,)? ] $(,)? ) => {
        str_replace_multiple!( str_replace!( $original , $aa , $ab ) , [ $( ( $ba , $bb , ) , )* ] , )
    }

}


mod handle;
pub use handle::*;

//mod instance;

//mod session;


struct EditorServerShared {
    commands_rx : Mutex<mpsc::Receiver<EditorCommand>>,
    database    : Arc<VoxidianDB>
}


pub struct EditorServer {
    display_game_address : String,
    commands_rx          : mpsc::Receiver<EditorCommand>,
    database             : Arc<VoxidianDB>
}
impl EditorServer {


    pub fn new(
        display_game_address : String,
        handle               : &mut EditorHandle,
        database             : Arc<VoxidianDB>
    ) -> Self {
        Self {
            display_game_address,
            commands_rx          : handle.commands_rx.take().expect("given `EditorHandle` is already controlled by an `EditorServer`"),
            database
        }
    }


    pub async fn run<A : ToSocketAddrs>(self, bind_addrs : A) -> Result<(), io::Error> {
        let app = Router::new();

        // Static assets
        let app = app.route("/robots.txt",                              routing::get(async || Self::route_asset(mime::TEXT, include_str!   ("assets/misc/robots.txt"                                           ).into_response())));
        let app = app.route("/assets/image/logo_transparent.png",       routing::get(async || Self::route_asset(mime::PNG,  include_bytes! ("assets/image/logo_transparent.png"                                ).into_response())));
        let app = app.route("/editor/voxidian_editor_frontend.js",      routing::get(async || Self::route_asset(mime::JS,   include_str!   ("../voxidian-editor-frontend/pkg/voxidian_editor_frontend.js"      ).into_response())));
        let app = app.route("/editor/voxidian_editor_frontend_bg.wasm", routing::get(async || Self::route_asset(mime::WASM, include_bytes! ("../voxidian-editor-frontend/pkg/voxidian_editor_frontend_bg.wasm" ).into_response())));

        // Root
        let app = app.route("/", routing::get(Html(include_str!("assets/template/root.html").replace("{{DISPLAY_GAME_ADDRESS}}", &self.display_game_address))));

        // Editor
        const EDITOR : &'static str = str_replace_multiple!( include_str!("assets/template/editor.html"), [
            ("{{VOXIDIAN_EDITOR_VERSION}}",      env!("CARGO_PKG_VERSION"           )),
            ("{{VOXIDIAN_EDITOR_COMMIT}}",       env!("VOXIDIAN_EDITOR_COMMIT"      )),
            ("{{VOXIDIAN_EDITOR_COMMIT_HASH}}",  env!("VOXIDIAN_EDITOR_COMMIT_HASH" ))
        ] );
        let app = app.route("/editor", routing::get(Html(EDITOR)));

        // Editor Websocket
        let app = app.route("/editor/ws", routing::any(Self::handle_editor_ws));

        // Fallback
        let app = app.fallback((StatusCode::NOT_FOUND, Html(include_str!("assets/template/404.html"))));

        // State
        let app = app.with_state(Arc::new(EditorServerShared {
            commands_rx : Mutex::new(self.commands_rx),
            database    : self.database
        }));

        // Run
        let listener = TcpListener::bind(bind_addrs).await?;
        axum::serve(listener, app.into_make_service()).await
    }


    fn route_asset(content_type : &'static str, data : impl IntoResponse) -> impl IntoResponse {
        let mut response = data.into_response();
        response.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static(content_type));
        response
    }


    async fn handle_editor_ws(
        upgrade : ws::WebSocketUpgrade,
        shared  : State<Arc<EditorServerShared>>
    ) -> impl IntoResponse {
        upgrade.protocols(["voxidian-editor"])
            .on_upgrade(move |socket| Self::handle_editor_socket(socket, shared.0))
    }

    async fn handle_editor_socket(
        mut socket  : ws::WebSocket,
            shared  : Arc<EditorServerShared>
    ) {

        let session_code = match (timeout(Duration::from_millis(2500), socket.recv()).await) {
            Err(_) => Err("Login took too long".into()),
            Ok(None) => Err("No session code".into()),
            Ok(Some(Err(err))) => Err(format!("An error occursed: {}", err).into()),
            Ok(Some(Ok(ws::Message::Binary(data)))) => {
                match (packet::decode::<packet::c2s::HandshakeC2SPacket>(data.as_ref())) {
                    Ok(handshake) => Ok(handshake.session_code),
                    Err(err) => Err(format!("An error occursed: {:?}", err).into())
                }
            },
            Ok(Some(Ok(_))) => Err("Bad data format".into())
        };
        let session_code = match (session_code) {
            Ok(session_code) => session_code,
            Err(reason) => {
                let _ = socket.send(ws::Message::Binary(packet::encode(packet::s2c::DisconnectS2CPacket { reason }).into())).await;
                return;
            }
        };

        voxidian_logger::warn!("{}", session_code);
        let _ = socket.send(ws::Message::Binary(packet::encode(packet::s2c::DisconnectS2CPacket { reason : "TODO".into() }).into())).await;

        /*let Some(mut session) = instances.get_pending_session(&session_code).await else {
            let _ = socket.send(ws::Message::Binary(packet::encode(packet::s2c::DisconnectS2CPacket {
                reason : "Invalid session code. Has it expired?".into()
            }).into()));
            return;
        };*/

        //session.activate(socket);
    }


}
