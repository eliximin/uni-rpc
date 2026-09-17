extern crate discord_rich_presence;
extern crate dotenvy;
extern crate tokio;

use discord_rich_presence::{DiscordIpc, DiscordIpcClient, activity, activity::Assets};
use dotenvy::dotenv;
use std::boxed::Box;
use std::env;
use std::error::Error;
use std::thread;
use std::time::Duration;
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

    let steam_api_key = env::var("STEAM_API").expect("Missing STEAM_API");
    let steamid64 = env::var("STEAMID64").expect("Missing STEAMID64");

    let appid = env::var("APP_ID").expect("Missing APP_ID");

    let (tx, mut rx) = mpsc::channel::<ActivityMetadata>(32);

    tokio::spawn(async move {
        let mut rpc_mgr = ipc_controller::IPCManager::new(&appid);

        let _ = rpc_mgr.ensure_connected();

        while let Some(state) = rx.recv().await {
            println!("received update: {:?}", state.details);

            if let Err(e) = rpc_mgr.update_presence(&state) {
                eprintln!("ipc errored: {e}");
                rpc_mgr.handle_disconnect();
            }
        }
    });

    let lfm_tx = tx.clone();

    tokio::spawn(async {
        if let Err(e) = lfmdaemon().await {
            eprintln!("Oops: {:?}", e);
        }
    });

    let steam_tx = tx.clone();

    tokio::spawn(async {
        if let Err(e) = steamdaemon(steam_tx, &steamid64, &steam_api_key).await {
            eprintln!("Oops: {:?}", e);
        }
    });

    // Due for a large refactor, honestly... I'll have to use startdaemon to supply this main script with assets and such. I'll branch the actual RPC module into a different script later.

    loop {
        thread::sleep(Duration::from_secs(10));
    }

    tokio::signal::ctrl_c().await?;
    println!("shutdown");

    Ok(())
}
