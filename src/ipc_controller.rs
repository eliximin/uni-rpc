extern crate discord_rich_presence;
extern crate dotenvy;
extern crate tokio;

use discord_rich_presence::{DiscordIpc, DiscordIpcClient, activity::Activity, activity::Assets};
use dotenvy::dotenv;
use std::boxed::Box;
use std::env;
use std::error::Error;

pub fn setup() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let appid = env::var("APP_ID").expect("Missing APP_ID");
    let mut manager = IPCManager::new(&appid);
    manager.ensure_connected()?;

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ActivityMetadata {
    pub name: Option<String>, // App title
    pub details: Option<String>,
    pub state: Option<String>,
    pub large_image: Option<String>,
    pub large_text: Option<String>, // Hover
    pub large_url: Option<String>,
    pub small_image: Option<String>,
    pub small_text: Option<String>, // Also hover
    pub small_url: Option<String>,
}

impl ActivityMetadata {
    pub fn has_assets(&self) -> bool {
        self.large_image.is_some()
            || self.large_text.is_some()
            || self.small_image.is_some()
            || self.small_text.is_some()
    }
}

pub struct IPCManager {
    client: Option<DiscordIpcClient>,
    app_id: String,
}

impl IPCManager {
    pub fn new(app_id: &str) -> Self {
        Self {
            client: None,
            app_id: app_id.to_string(),
        }
    }

    fn ensure_connected(&mut self) -> Result<&mut DiscordIpcClient, Box<dyn Error>> {
        if self.client.is_none() {
            let mut client = DiscordIpcClient::new(&self.app_id);
            client.connect()?;
            self.client = Some(client);
        }
        Ok(self.client.as_mut().unwrap())
    }

    pub fn handle_disconnect(&mut self) {
        if let Some(mut client) = self.client.take() {
            let _ = client.close();
        }
    }

    pub fn update_presence(&mut self, state: &ActivityMetadata) -> Result<(), Box<dyn Error>> {
        let client = self.ensure_connected()?;

        let mut payload = Activity::new();

        if let Some(d) = &state.details {
            payload = payload.details(d);
        }
        if let Some(s) = &state.state {
            payload = payload.state(s);
        }

        if state.has_assets() {
            let mut assets = Assets::new();
            if let Some(img) = &state.large_image {
                assets = assets.large_image(img);
            }
            if let Some(txt) = &state.large_text {
                assets = assets.large_text(txt);
            }
            if let Some(img) = &state.small_image {
                assets = assets.small_image(img);
            }
            if let Some(txt) = &state.small_text {
                assets = assets.small_text(txt);
            }
            payload = payload.assets(assets);
        }

        client.set_activity(payload)?;
        Ok(())
    }
}
