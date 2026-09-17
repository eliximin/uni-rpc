extern crate dotenvy;

use dotenvy::dotenv;
use reqwest::{self, StatusCode};
use scraper::{Html, Selector};
use serde::Deserialize;
use tokio::sync::mpsc;
use std::boxed::Box;
use std::env;
use std::error::Error;
use std::time::Duration;

use crate::ipc_controller::ActivityMetadata;

pub async fn steamdaemon(
    tx: mpsc::Sender<ActivityMetadata>,
    steamid64: &str,
    apikey: &str,
) -> Result<(), Box<dyn Error>> {
    let rate = 10; // Every 10 seconds, makes a request. This is made to prevent rate limits because rate limits reek.




    let steamid3 = steamid64.parse::<u64>().unwrap() - 76561197960265728;
    println!("SteamID3: {}\nSteamID64: {}", steamid3, steamid64);

    let client = reqwest::Client::builder()
        .user_agent("UniRPC-ALPHA/0.1 github/eliximin") // As demanded by... well, literally every single endpoint. Makes the client recognizable
        .build()?;

    loop {
        let url = format!(
            "https://api.steampowered.com/ISteamUser/GetPlayerSummaries/v0002/?key={apikey}&steamids={steamid64}"
        );

        let result = client.get(&url).send().await?;

        let status = result.status();
        let data: SteamResponse = result.json().await?;

        let steamuser = data
            .response
            .player
            .first()
            .filter(|p| p.is_in_game())
            .cloned();

        if let Some(user) = &steamuser {
            if user.is_in_game() {
                println!(
                    "{} is playing {:?}, id is {}",
                    user.name,
                    user.gamedetails.as_deref().unwrap_or("Unknown"),
                    user.gameid
                );
            }
        } else {
            continue;
        }

        println!("Status: {}", status);

        let miniurl = format!("https://steamcommunity.com/miniprofile/{steamid3}");

        let rpcfetch = client.get(&miniurl).send().await?;
        println!("RPC Status: {}", rpcfetch.status());
        let rpcbody = rpcfetch.text().await?;

        let (game_name, rich_presence): (Option<String>, Option<String>) = {
            let document = Html::parse_document(&rpcbody);

            let game_sel = Selector::parse("span.miniprofile_game_name").ok();
            let rp_sel = Selector::parse("span.rich_presence").ok();

            let game = game_sel
                .and_then(|sel| {
                    document
                        .select(&sel)
                        .next()
                        .map(|el| el.text().collect::<String>())
                })
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());

            let rp = rp_sel
                .and_then(|sel| {
                    document
                        .select(&sel)
                        .next()
                        .map(|el| el.text().collect::<String>())
                })
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());

            (game, rp)
        };

        println!("RPC: {:?}", rich_presence);

        if game_name != steamuser.unwrap().gamedetails {
            println!("Inequal!")
        }

        tokio::time::sleep(Duration::from_secs(rate)).await;

        match status {
            StatusCode::OK => {
                println!("Proceed!");
            }
            _ => {
                println!("End of steam loop.");
                continue;
            }
        }

    }

    println!("Something stopped the loop. Check above?"); // Intentionally unreachable code, since the loop shouldn't stop by now

    Ok(())
}

// Eases JSON stuff and defines it as a struct for future use
#[derive(Deserialize, Debug)]
pub struct SteamResponse {
    pub response: Response,
}

#[derive(Deserialize, Debug)]
pub struct Response {
    #[serde(rename = "players")]
    pub player: Vec<Player>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Player {
    #[serde(rename = "personaname")]
    pub name: String,
    /*    pub avatarsmall: String,
    pub avatarmedium: String,
    pub avatarfull: String,
    pub timecreated: Number,*/ // These are commented because I'd like to silence the errors since I'm not using them right now
    #[serde(rename = "gameextrainfo")]
    pub gamedetails: Option<String>,
    pub gameid: String,
}

impl Player {
    pub fn is_in_game(&self) -> bool {
        self.gamedetails.is_some() // Checks if in game
    }
}
