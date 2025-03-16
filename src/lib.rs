#![feature(

    // Syntax
    decl_macro

)]


use voxidian_logger::{ debug, info, error };
use axecs::prelude::*;
use axecs::future::UntilExitFuture;
use std::io;
use std::net::SocketAddr;
use tokio::net::{ self, ToSocketAddrs };


pub mod webserver;

pub mod peer;

pub mod instances;

mod util;


pub struct EditorPlugin {
    bind_addrs        : Vec<SocketAddr>,
    display_game_addr : String
}

impl EditorPlugin {

    pub async fn new<A : ToSocketAddrs>(
        bind_addrs        : A,
        display_game_addr : String
    ) -> io::Result<Self> { Ok(Self {
        bind_addrs        : net::lookup_host(bind_addrs).await?.collect(),
        display_game_addr
    }) }

}

impl Plugin for EditorPlugin {
    fn build(self, app : &mut App) {

        app.add_systems(Startup, run_webserver.pass((self.bind_addrs, self.display_game_addr)));

        app.add_systems(Cycle, instances::read_instance_events);
        app.add_systems(Cycle, instances::session::read_session_events);
        app.add_systems(Cycle, instances::session::update_state);

    }
}


async fn run_webserver(
    In((bind_addrs, display_game_addr)) : In<(Vec<SocketAddr>, String)>,
    cmds                                : Commands
) {

    info!("Starting editor server...");
    match (UntilExitFuture::new(cmds.clone(), webserver::run(
        cmds.clone(),
        bind_addrs.as_slice(),
        &display_game_addr
    )).await) {
        Some(Err(err)) => {
            error!("Failed to start editor server: {}", err);
            cmds.exit(AppExit::Err(err.into()));
        },
        _ => {
            debug!("Shut down editor server.");
        }
    }

}
