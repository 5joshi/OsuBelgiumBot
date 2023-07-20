use std::sync::Arc;

use chrono::Duration;
use twilight_model::{application::interaction::ApplicationCommand, id::UserId};

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
    //? Do this better
    if !(matches!(command.user_id(), Ok(UserId(185773647246524416)))
        || matches!(command.user_id(), Ok(UserId(219905108316520448)))
        || matches!(command.user_id(), Ok(UserId(127449164224266240)))
        || matches!(command.user_id(), Ok(UserId(250998032856776704))))
    {
        let builder =
            MessageBuilder::new().error("You do not have permission to use this command!");
        return command.create_message(&ctx, builder).await;
    }
    info!("Adding map to osuvs queue...");
    match map_id {
        Some(id) => {
            let start_date = ctx.database.get_latest_osuvs_date().await?;
            let end_date = start_date + Duration::days(7);
            let _ = ctx
                .database
                .insert_osuvs_map(id, start_date, end_date)
                .await?;
            let builder = MessageBuilder::new()
                .embed("Added the map! Big chungy boing!")
                .ephemeral();
            command.create_message(&ctx, builder).await
        }
        None => {
            let builder = MessageBuilder::new().error("There is currently no ongoing OsuVS!");
            command.create_message(&ctx, builder).await
        }
    }
}
