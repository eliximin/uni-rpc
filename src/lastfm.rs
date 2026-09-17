extern crate dotenvy;

use dotenvy::dotenv;
use reqwest::{self, StatusCode};
use serde::Deserialize;
use std::boxed::Box;
use std::error::Error;
use std::time::Duration;
use tokio::sync::mpsc;

use crate::ipc_controller::{ActivityMetadata, IpcType};

pub async fn lfmdaemon(
    tx: mpsc::Sender<ActivityMetadata>,
    lastfm_name: &str,
    lastfm_api_key: &str,
) -> Result<(), Box<dyn Error>> {
    let rate = 10; // Every 10 seconds, makes a request. This is made to prevent rate limits because rate limits reek.

    dotenv().ok();



    let client = reqwest::Client::builder()
        .user_agent("UniRPC-ALPHA/0.1 github/eliximin")
        .build()?;

    loop {

        tokio::time::sleep(Duration::from_secs(rate)).await;

        let url = format!(
            "https://ws.audioscrobbler.com/2.0/?method=user.getRecentTracks&api_key={lastfm_api_key}&user={lastfm_name}&format=json"
        );

        let result = client.get(&url).send().await?;

        let status = result.status();
        let data: LastFMResponse = result.json().await?;

        let first_track = data.recenttracks.track.first();

        if let Some(track) = first_track
            && first_track.clone().unwrap().is_now_playing()
        {
            println!(
                "Playing {:?} by {:?}",
                track.name, track.artist.name
            );
            println!("Album is {}", track.album.name);

        }
        else {
            continue
        }

        let current_track = first_track.unwrap();

        let large_image_url = current_track.image
            .iter()
            .find(|img| img.size == "extralarge")
            .or_else(|| current_track.image.iter().find(|img| img.size == "large"))
            .or_else(|| current_track.image.first())
            .map(|img| &img.url);

        let state = ActivityMetadata { activity_type: Some(IpcType::Listening), name: Some(current_track.artist.name.clone()), details: current_track.name.clone(), state: Some(current_track.artist.name.clone()), large_image: large_image_url.cloned(), large_text: Some(current_track.album.name.clone()), large_url: current_track.url.clone(), small_image: None, small_text: None, small_url: None };

        println!("Status: {}", status);



        tx.send(state).await?;



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
    pub name: Option<String>,
    pub artist: Artist,
    pub album: Album,
    pub image: Vec<Image>,
    #[serde(rename = "@attr")]
    pub attr: Option<TrackAttr>,
    pub url: Option<String>,
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
pub struct Image {
    size: String,
    #[serde(rename = "#text", default)]
    pub url: String,
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
