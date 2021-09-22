use std::sync::Arc;

use twilight_model::application::{
    callback::{CallbackData, InteractionResponse},
    interaction::ApplicationCommand,
};

use crate::{context::Context, error::BotResult};

#[command]
#[description = "Play ping pong with the bot"]
pub struct Ping;

async fn ping(ctx: Arc<Context>, command: ApplicationCommand) -> BotResult<()> {
    let response = InteractionResponse::ChannelMessageWithSource(CallbackData {
        allowed_mentions: None,
        components: None,
        content: Some("BIG CHUNGUS".to_string()),
        embeds: vec![],
        flags: None,
        tts: None,
    });

    ctx.http
        .interaction_callback(command.id, &command.token, &response)
        .exec()
        .await?;

    Ok(())
}
