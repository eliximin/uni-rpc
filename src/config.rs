extern crate inquire;
extern crate keyring;

use inquire::{Password, PasswordDisplayMode, Select, Text};
use keyring::Entry;

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

pub fn get_info() -> Result<(String, String, String, String), Box<dyn std::error::Error>> {
    let appid = Password::new("Discord App ID")
        .with_display_mode(PasswordDisplayMode::Masked)
        .with_help_message(
            "https://discord.com/developers/applications, make an app and copy the ID",
        )
        .prompt()?;

    let current_session =
        Select::new("LastFM or Steam?", vec!["LastFM", "Steam"])
            .with_help_message("I'm too lazy to actually stop you from filling out either of those fields, so just don't fill out Steam-related ones if you pick LastFM. And vice-versa.")
            .prompt()?;

    let lfm_name = Text::new("What's your name on LastFM?").prompt()?;

    let steamid64 = Text::new("What's your SteamID64?")
        .with_help_message("Use https://steamid.io/ to get it.")
        .prompt()?;



    Ok((appid, lfm_name, steamid64, current_session.to_string()))
}

pub fn please_speed_i_need_keys() -> Result<(String, String, String), Box<dyn std::error::Error>> {
    let already_filled =
        Select::new("Have you already input your API keys before?", vec!["Yes", "No."]).prompt()?;

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
        .with_help_message("you also need the other 2 api keys for steam and steamgriddb")
        .prompt()?;

    let steam_key = Password::new("Steam api key")
        .with_display_mode(PasswordDisplayMode::Masked)
        .prompt()?;

    let steamgriddb_key = Password::new("Steamgriddb api key")
        .with_display_mode(PasswordDisplayMode::Masked)
        .prompt()?;

    set_secret(ApiKeys::LastFm.keyname(), &lfm_key)?;
    set_secret(ApiKeys::Steam.keyname(), &steam_key)?;
    set_secret(ApiKeys::GridDB.keyname(), &steamgriddb_key)?;

    Ok((lfm_key, steam_key, steamgriddb_key))
}
