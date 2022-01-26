use crate::{database::Database, error::BotResult};

use futures::StreamExt;
use hashbrown::HashMap;
use twilight_model::id::UserId;

impl Database {
    pub async fn get_manual_links(&self) -> BotResult<HashMap<UserId, u32>> {
        let mut stream = sqlx::query!("SELECT * FROM manual_links;").fetch(&self.pool);
        let mut manual_links = HashMap::new();
        while let Some(entry) = stream.next().await.transpose()? {
            let discord_id = UserId(entry.discord_id as u64);
            let osu_id = entry.osu_id as u32;
            manual_links.insert(discord_id, osu_id);
        }
        Ok(manual_links)
    }
}
