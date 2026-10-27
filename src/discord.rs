use anyhow::Result;
use discord_rich_presence::{
    activity::{Activity, ActivityType, Assets, Timestamps},
    DiscordIpc, DiscordIpcClient,
};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct DiscordRpc {
    client: DiscordIpcClient,
}

impl DiscordRpc {
    pub fn from_environment() -> Option<Self> {
        let client_id = std::env::var("ANILY_DISCORD_CLIENT_ID").ok()?;
        if client_id.trim().is_empty() {
            return None;
        }

        let mut client = DiscordIpcClient::new(&client_id);
        client.connect().ok()?;
        Some(Self { client })
    }

    pub fn set_episode(
        &mut self,
        title: &str,
        episode: &str,
        cover_asset: Option<&str>,
    ) -> Result<()> {
        let mut assets = Assets::new().large_text("Watching with Anily");
        if let Some(asset) = cover_asset {
            assets = assets.large_image(asset);
        }
        let start = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;
        let activity = Activity::new()
            .name("Anily")
            .activity_type(ActivityType::Watching)
            .details(title)
            .state(format!("Episode {episode}"))
            .timestamps(Timestamps::new().start(start))
            .assets(assets);
        self.client.set_activity(activity)?;
        Ok(())
    }

    pub fn clear(&mut self) -> Result<()> {
        self.client.clear_activity()?;
        Ok(())
    }
}

impl Drop for DiscordRpc {
    fn drop(&mut self) {
        let _ = self.client.clear_activity();
        let _ = self.client.close();
    }
}
