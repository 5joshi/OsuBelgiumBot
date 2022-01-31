use chrono::{DateTime, Duration, Utc};
use futures::StreamExt;
use hashbrown::HashMap;
use rosu_v2::{model::score, prelude::Score};
use sqlx::Row;
use twilight_model::{channel::Message, id::ChannelId};

use crate::{commands::MessageActivity, database::Database, error::BotResult};

impl Database {
    pub async fn get_curr_osuvs_map(&self) -> Option<(u32, DateTime<Utc>, DateTime<Utc>)> {
        sqlx::query!("SELECT * FROM osuvs_maps WHERE start_date < now() AND end_date > now();")
            .fetch_optional(&self.pool)
            .await
            .map_or_else(
                |why| {
                    unwind_error!(warn, why, "Couldn't retrieve osuvs map: {}");
                    None
                },
                |result| {
                    result.map(|r| {
                        (
                            r.beatmap_id as u32,
                            r.start_date as chrono::DateTime<Utc>,
                            r.end_date as chrono::DateTime<Utc>,
                        )
                    })
                },
            )
    }

    // Return a HashMap mapping user ids to HashMaps mapping mods to scores
    pub async fn get_osuvs_highscores(&self) -> BotResult<HashMap<u32, Score>> {
        let map_id = self.get_curr_osuvs_map().await.map(|t| t.0);
        match map_id {
            Some(id) => {
                let mut stream = sqlx::query!(
                    "SELECT user_id, score FROM osuvs_scores WHERE beatmap_id = $1;",
                    id as i32
                )
                .fetch(&self.pool);
                let mut scores: HashMap<u32, Score> = HashMap::new();
                while let Some(r) = stream.next().await.transpose()? {
                    let (user_id, score) = (r.user_id as u32, r.score);

                    scores.insert(user_id, serde_json::from_value(score)?);
                }
                return Ok(scores);
            }
            None => return Ok(HashMap::new()),
        }
    }

    //? Do I need bool return type
    pub async fn insert_osuvs_highscores(
        &self,
        map_id: u32,
        user: u32,
        score: Score,
    ) -> BotResult<bool> {
        let query = sqlx::query!(
            "INSERT INTO osuvs_scores (beatmap_id, user_id, score) VALUES ($1, $2, $3) ON CONFLICT (beatmap_id, user_id) DO UPDATE SET score = $3;",
            map_id as i32,
            user as i32,
            serde_json::to_value(&score)?
        );
        let result = query.execute(&self.pool).await?;
        Ok(result.rows_affected() == 1)
    }

    //? Do I need bool return type
    pub async fn insert_osuvs_map(
        &self,
        map_id: u32,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> BotResult<bool> {
        let query = sqlx::query!(
            "INSERT INTO osuvs_maps (beatmap_id, start_date, end_date) VALUES ($1, $2, $3) ON CONFLICT (beatmap_id, start_date, end_date) DO NOTHING;",
            map_id as i32,
            start_date,
            end_date
        );
        let result = query.execute(&self.pool).await?;
        Ok(result.rows_affected() == 1)
    }

    pub async fn get_latest_osuvs_date(&self) -> BotResult<DateTime<Utc>> {
        let now = Utc::now();
        sqlx::query!("SELECT end_date FROM osuvs_maps ORDER BY end_date ASC LIMIT 1")
            .fetch_optional(&self.pool)
            .await
            .map(|r_opt| match r_opt {
                Some(r) => r.end_date.max(now),
                None => now,
            })
            .map_err(From::from)
    }
}
