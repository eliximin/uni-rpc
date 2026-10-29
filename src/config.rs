extern crate inquire;
extern crate keyring;

use std::error::Error;

use inquire::{Password, PasswordDisplayMode, Select, Text};
use keyring::Entry;

use serde::{Deserialize, Serialize};

use crate::ipc_controller::{ActivityMetadata, IpcType};

pub enum ApiKeys {
    LastFm,
    Steam,
    GridDB,
}

impl ApiKeys {
    pub fn keyname(&self) -> &'static str {
        match self {
            ApiKeys::LastFm => "lastfm_api_key",
            ApiKeys::Steam => "steam_api_key",
            ApiKeys::GridDB => "griddb_api_key",
        }
    }
}

pub fn set_secret(key_name: &str, secret: &str) -> Result<(), keyring::Error> {
    let entry = Entry::new("uni-rpc", key_name)?;
    entry.set_password(secret)?;
    Ok(())
}

pub fn get_secret(key_name: &str) -> Option<String> {
    let entry = Entry::new("uni-rpc", key_name).ok()?;
    match entry.get_password() {
        Ok(password) => Some(password),
        Err(err) => {
            eprintln!("Oops! {}", err);
            None
        }
    }
}

pub fn get_info() -> Result<(String, String, String), Box<dyn std::error::Error>> {
    let current_session = Select::new("LastFM or Steam?", vec!["LastFM", "Steam"])
        .with_help_message("This'll be phased out eventually.")
        .prompt()?;

    let already_filled = Select::new(
        "Did you already provide your SteamID64 and Last.FM username?",
        vec!["Yes, skip this.", "No, let me refill them."],
    )
    .prompt()?;

    let is_filled = match already_filled {
        "Yes, skip this." => true,
        "No, let me refill them." => false,
        _ => unreachable!(),
    };

    if is_filled {
        let current_cfg: ReqInfo = confy::load("uni-rpc", "metadata")?;
        return Ok((
            current_cfg.lastfm_name,
            current_cfg.steamid64,
            current_session.to_string(),
        ));
    }

    let lfm_name = Text::new("What's your username on LastFM?").prompt()?;

    let steamid64 = Text::new("What's your SteamID64?")
        .with_help_message("Use https://steamid.io/ to get it.")
        .prompt()?;

    let new_cfg = ReqInfo {
        lastfm_name: lfm_name.clone(),
        steamid64: steamid64.clone(),
    };

    confy::store("uni-rpc", "metadata", new_cfg)?;

    Ok((lfm_name, steamid64, current_session.to_string()))
}

pub fn get_vars() -> Result<Variables, Box<dyn std::error::Error>> {
    let already_filled = Select::new(
        "Do you want to edit your RPC variables?",
        vec!["No, skip this.", "Yes, let me edit them."],
    )
    .with_help_message(
        "This will.. reset your entire variable config. You'll have to redo this again.",
    )
    .prompt()?;

    let is_filled = match already_filled {
        "No, skip this." => true,
        "Yes, let me edit them." => false,
        _ => unreachable!(),
    };

    if is_filled {
        let current_cfg: Variables = confy::load("uni-rpc", "variables")?;
        return Ok(current_cfg);
    }

    let activity = Select::new(
        "What do you want to set the activity type as?",
        vec!["Playing", "Watching", "Listening", "Competing"],
    )
    .with_help_message("I'm sure this needs no explanation.")
    .prompt()?;

    let activitytype = match activity {
        "Playing" => IpcType::Playing,
        "Watching" => IpcType::Watching,
        "Listening" => IpcType::Listening,
        "Competing" => IpcType::Competing,
        _ => unreachable!(),
    };

    let name = Text::new("What do you want to set the name as?")
        .with_help_message("Listening to X, Playing Y.")
        .prompt()?;

    let details = Text::new("What do you want to set the details as?")
        .with_help_message("The first line, right below the activity name.")
        .prompt()?;

    let state = Text::new("What do you want to set the state as?")
        .with_help_message("The middle line.")
        .prompt()?;

    let large_image = Text::new("What do you want to set the large image as?")
        .with_help_message("Can be a URL, or an asset key defined in your developer portal for the app. Link cannot be more than 256 characters.")
        .prompt()?;

    let large_text = Text::new("What do you want to set the large image's hover text to be?")
        .with_help_message("Text. Yeah.")
        .prompt()?;

    let small_image = Text::new("What do you want to set the small image as?")
        .with_help_message("Same as large image. Shows up as an image in the bottom right of the large image. Link cannot be more than 256 characters.")
        .prompt()?;

    let small_text = Text::new("What do you want to set the small image's hover text to be?")
        .with_help_message("Yup. Text. Yeah.")
        .prompt()?;

    let full = ActivityMetadata {
        activity_type: Some(activitytype),
        src: "Uni".to_string(),
        name: Some(name),
        details: Some(details),
        state: Some(state),
        large_image: Some(large_image),
        large_text: Some(large_text),
        small_image: Some(small_image),
        small_text: Some(small_text),
        small_url: None,
        large_url: None,
    };

    let var_js = Variables {
        variables: serde_json::to_string(&full).unwrap(),
    };

    confy::store("hhh", "variables", &var_js)?;

    Ok(var_js)
}

pub fn please_speed_i_need_keys() -> Result<(String, String, String), Box<dyn Error>> {
    let already_filled = Select::new(
        "Have you already input your API keys before?",
        vec!["Yes", "No."],
    )
    .prompt()?;

    let is_filled = match already_filled {
        "Yes" => true,
        "No." => false,
        _ => unreachable!(),
    };

    if is_filled {
        let lfm_key = get_secret(ApiKeys::LastFm.keyname()).expect("no LFM key");
        let steam_key = get_secret(ApiKeys::Steam.keyname()).expect("no steam key");
        let steamgriddb_key = get_secret(ApiKeys::GridDB.keyname()).expect("no griddb key");

        return Ok((lfm_key, steam_key, steamgriddb_key));
    }

    let lfm_key = Password::new("Last.FM api key")
        .with_display_mode(PasswordDisplayMode::Masked)
        .with_help_message(
            "Sign into last.fm and create an API key at https://www.last.fm/api/account/create",
        )
        .prompt()?;

    let steam_key = Password::new(
        "Sign into Steam and head to https://steamcommunity.com/dev/apikey to creat an API key.",
    )
    .with_display_mode(PasswordDisplayMode::Masked)
    .prompt()?;

    let steamgriddb_key = Password::new("Head to steamgriddb.com, sign in, and make an API key over at https://www.steamgriddb.com/profile/preferences/api")
        .with_display_mode(PasswordDisplayMode::Masked)
        .prompt()?;

    set_secret(ApiKeys::LastFm.keyname(), &lfm_key)?;
    set_secret(ApiKeys::Steam.keyname(), &steam_key)?;
    set_secret(ApiKeys::GridDB.keyname(), &steamgriddb_key)?;

    for i in [&lfm_key, &steam_key, &steamgriddb_key] {
        if i.is_empty() {
            panic!("Make sure your api keys are filled!")
        };
    }

    Ok((lfm_key, steam_key, steamgriddb_key))
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct ReqInfo {
    lastfm_name: String,
    steamid64: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Variables {
    pub variables: String,
}
