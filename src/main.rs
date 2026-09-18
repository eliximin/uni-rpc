extern crate discord_rich_presence;
extern crate dotenvy;
extern crate tokio;

use discord_rich_presence::DiscordIpc;
use dotenvy::dotenv;
use std::boxed::Box;
use std::env;
use std::error::Error;
use tokio::sync::mpsc;

mod ipc_controller;
mod lastfm;
mod steam;
use crate::ipc_controller::ActivityMetadata;
use crate::lastfm::lfmdaemon;
use crate::steam::steamdaemon;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let appid = env::var("APP_ID").expect("Missing APP_ID");

    let steam_api_key = env::var("STEAM_API").expect("Missing STEAM_API");
    let steamid64 = env::var("STEAMID64").expect("Missing STEAMID64");
    let gridapi = env::var("GRIDDB_API").expect("Missing GRIDDB_API");

    let lastfm_api_key = env::var("LASTFM_API").expect("Missing LASTFM_API");
    let lastfm_name = env::var("LASTFM_NAME").expect("Missing LASTFM_NAME");

    let (tx, mut rx) = mpsc::channel::<ActivityMetadata>(32);

    tokio::spawn(async move {
        let mut rpc_mgr = ipc_controller::IPCManager::new(&appid);

        let _ = rpc_mgr.ensure_connected();

        while let Some(state) = rx.recv().await {
            println!("received update: {:?}", state.name);

            if let Err(e) = rpc_mgr.update_presence(&state) {
                eprintln!("ipc errored: {e}");
                rpc_mgr.handle_disconnect();
            }
        }
    });
    /*

    let lfm_tx = tx.clone();

    tokio::spawn(async move {
        if let Err(e) = lfmdaemon(lfm_tx, &lastfm_name, &lastfm_api_key).await {
            eprintln!("Oops: {:?}", e);
        }
    });*/

    let steam_tx = tx.clone();

    let steamid = steamid64.clone();
    let steamapikey = steam_api_key.clone();
    let griddbapi = gridapi;

    tokio::spawn(async move {
        if let Err(e) = steamdaemon(steam_tx, &steamid, &steamapikey, &griddbapi).await {
            eprintln!("Oops: {:?}", e);
        }
    });
    // Due for a large refactor, honestly... I'll have to use startdaemon to supply this main script with assets and such. I'll branch the actual RPC module into a different script later.

    tokio::signal::ctrl_c().await?;
    println!("shutdown");

    Ok(())
}
