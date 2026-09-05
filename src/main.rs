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

mod ipc_controller;
mod lastfm;
mod steam;
use crate::lastfm::lfmdaemon;
use crate::steam::steamdaemon;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let appid = env::var("APP_ID").expect("Missing APP_ID");

    let mut client = DiscordIpcClient::new(appid);

    tokio::spawn(async {
        if let Err(e) = lfmdaemon().await {
            eprintln!("Oops: {:?}", e);
        }
    });

    tokio::spawn(async {
        if let Err(e) = steamdaemon().await {
            eprintln!("Oops: {:?}", e);
        }
    });

    client.connect()?;
    // Due for a large refactor, honestly... I'll have to use startdaemon to supply this main script with assets and such. I'll branch the actual RPC module into a different script later.

    let current_assets = Assets::new().large_image("https://thumb.wikimedia.org/wikipedia/commons/thumb/8/8a/Banana-Single.jpg/960px-Banana-Single.jpg?_=20150318233437");

    let payload = activity::Activity::new()
        .name("test")
        .state("test :P")
        .details("details?")
        .assets(current_assets);
    client.set_activity(payload)?;

    loop {
        thread::sleep(Duration::from_secs(10));
    }
}
