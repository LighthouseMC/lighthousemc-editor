use crate::util::str_replace_multiple;
use voxidian_logger::pass;
use axecs::prelude::*;
use std::io;
use tokio::net::{ TcpListener, ToSocketAddrs };
use axum::{ Router, routing };
use axum::http::{ StatusCode, HeaderValue };
use axum::http::header::CONTENT_TYPE;
use axum::response::{ IntoResponse, Html };
use axum::extract::State;
use axum::extract::ws::WebSocketUpgrade;


mod mime {
    pub const TEXT : &'static str = "text/plain";
    pub const PNG  : &'static str = "image/png";
    pub const JS   : &'static str = "application/javascript";
    pub const WASM : &'static str = "application/wasm";
}


pub async fn run<A : ToSocketAddrs>(
    cmds                 : Commands,
    bind_addrs           : A,
    display_game_address : &str
) -> Result<(), io::Error> {
    let app = Router::new();

    // Static assets
    let app = app.route("/robots.txt",                              routing::get(async || route_asset(mime::TEXT, include_str!   ("assets/misc/robots.txt"                                           ).into_response())));
    let app = app.route("/assets/image/logo_transparent.png",       routing::get(async || route_asset(mime::PNG,  include_bytes! ("assets/image/logo_transparent.png"                                ).into_response())));
    let app = app.route("/editor/voxidian_editor_frontend.js",      routing::get(async || route_asset(mime::JS,   include_str!   ("../voxidian-editor-frontend/pkg/voxidian_editor_frontend.js"      ).into_response())));
    let app = app.route("/editor/voxidian_editor_frontend_bg.wasm", routing::get(async || route_asset(mime::WASM, include_bytes! ("../voxidian-editor-frontend/pkg/voxidian_editor_frontend_bg.wasm" ).into_response())));

    // Root
    let app = app.route("/", routing::get(Html(include_str!("assets/template/root.html").replace("{{DISPLAY_GAME_ADDRESS}}", display_game_address))));

    // Editor
    const EDITOR : &'static str = str_replace_multiple!( include_str!("assets/template/editor.html"), [
        ("{{VOXIDIAN_EDITOR_VERSION}}",      env!("CARGO_PKG_VERSION"           )),
        ("{{VOXIDIAN_EDITOR_COMMIT}}",       env!("VOXIDIAN_EDITOR_COMMIT"      )),
        ("{{VOXIDIAN_EDITOR_COMMIT_HASH}}",  env!("VOXIDIAN_EDITOR_COMMIT_HASH" ))
    ] );
    let app = app.route("/editor", routing::get(Html(EDITOR)));

    // Editor Websocket
    let app = app.route("/editor/ws", routing::any(handle_editor_websocket));

    // Fallback
    let app = app.fallback((StatusCode::NOT_FOUND, Html(include_str!("assets/template/404.html"))));

    // state
    let app = app.with_state(cmds);

    // Run
    let listener = TcpListener::bind(bind_addrs).await?;
    pass!("Started editor server.");
    axum::serve(listener, app.into_make_service()).await
}


fn route_asset(content_type : &'static str, data : impl IntoResponse) -> impl IntoResponse {
    let mut response = data.into_response();
    response.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static(content_type));
    response
}


async fn handle_editor_websocket(
    upgrade : WebSocketUpgrade,
    cmds    : State<Commands>
) -> impl IntoResponse {
    upgrade.protocols(["voxidian-editor"])
        .on_upgrade(async move |socket| crate::peer::handle_editor_websocket(cmds.0, socket).await)
}
