use std::sync::Arc;

use chrono::Duration;
use twilight_model::application::interaction::ApplicationCommand;

use crate::{
    context::Context,
    error::BotResult,
    utils::{ApplicationCommandExt, MessageBuilder},
};

pub async fn start(
    ctx: Arc<Context>,
    command: ApplicationCommand,
    map_id: Option<u32>,
) -> BotResult<()> {
    match map_id {
        Some(id) => {
            let start_date = ctx.database.get_latest_osuvs_date().await?;
            let end_date = start_date + Duration::days(7);
            let _ = ctx
                .database
                .insert_osuvs_map(id, start_date, end_date)
                .await?;
            Ok(())
        }
        None => {
            let builder = MessageBuilder::new().error("There is currently no ongoing OsuVS!");
            command.create_message(&ctx, builder).await
        }
    }
}
