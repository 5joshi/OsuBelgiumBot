use chrono::{DateTime, Duration, Utc};
use futures::StreamExt;
use sqlx::Row;
use twilight_model::{channel::Message, id::ChannelId};

use crate::{database::Database, error::BotResult};

impl Database {
    //? Do I need bool return type
    pub async fn insert_message(&self, message: &Message) -> BotResult<bool> {
        let query = sqlx::query!(
            "INSERT INTO messages (id, channel_id, author, content, timestamp, bot) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (id) DO NOTHING;",
            message.id.0 as i64,
            message.channel_id.0 as i64,
            message.author.id.0 as i64,
            message.content,
            message.timestamp.parse::<DateTime<Utc>>()?,
            message.author.bot
        );
        let result = query.execute(&self.pool).await?;
        Ok(result.rows_affected() == 1)
    }
}
