extern crate dotenvy;

use dotenvy::dotenv;
use reqwest::{self, StatusCode};
use serde::Deserialize;
use std::boxed::Box;
use std::env;
use std::error::Error;
use std::time::Duration;

pub async fn lfmdaemon() -> Result<(), Box<dyn Error>> {
    let rate = 10; // Every 10 seconds, makes a request. This is made to prevent rate limits because rate limits reek.

    dotenv().ok();

    let lastfm_api_key = env::var("LASTFM_API").expect("Missing LASTFM_API");

    let client = reqwest::Client::builder()
        .user_agent("UniRPC-ALPHA/0.1 github/eliximin")
        .build()?;

    loop {
        let url = format!(
            "https://ws.audioscrobbler.com/2.0/?method=user.getRecentTracks&api_key={lastfm_api_key}&user=eliximinatus&format=json"
        );

        let result = client.get(&url).send().await?;

        let status = result.status();
        let data: LastFMResponse = result.json().await?;

        if let Some(first_track) = data.recenttracks.track.first()
            && first_track.is_now_playing()
        {
            println!(
                "Playing {} by {}",
                first_track.name, first_track.artist.name
            );
            println!("Album is {}", first_track.album.name);
        }

        println!("Status: {}", status);

        tokio::time::sleep(Duration::from_secs(rate)).await;

        match status {
            StatusCode::OK => {
                println!("Proceed!");
                continue;
            }
            _ => {
                println!("Break.");
                break;
            }
        }
    }

    println!("Something stopped the loop. Check above?");

    Ok(())
}

#[derive(Deserialize, Debug)]
pub struct LastFMResponse {
    pub recenttracks: RecentTracks,
}

#[derive(Deserialize, Debug)]
pub struct RecentTracks {
    pub track: Vec<Track>,
}

#[derive(Deserialize, Debug)]
pub struct Track {
    pub name: String,
    pub artist: Artist,
    pub album: Album,
    #[serde(rename = "@attr")]
    pub attr: Option<TrackAttr>,
}

#[derive(Deserialize, Debug)]
pub struct TrackAttr {
    pub nowplaying: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Artist {
    #[serde(rename = "#text")]
    pub name: String,
}

#[derive(Deserialize, Debug, Default)]
pub struct Album {
    #[serde(rename = "#text", default)]
    pub name: String,
}

impl Track {
    pub fn is_now_playing(&self) -> bool {
        self.attr.as_ref().and_then(|a| a.nowplaying.as_deref()) == Some("true")
    }
}
