extern crate inquire;
extern crate keyring;

use std::error::Error;

use inquire::{Password, PasswordDisplayMode, Select, Text};
use keyring::Entry;

use serde::{Deserialize, Serialize};

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
        let current_cfg: ReqInfo = confy::load("uni-rpc", None)?;
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

    confy::store("uni-rpc", None, new_cfg)?;

    Ok((lfm_name, steamid64, current_session.to_string()))
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

    Ok((lfm_key, steam_key, steamgriddb_key))
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct ReqInfo {
    lastfm_name: String,
    steamid64: String,
}
