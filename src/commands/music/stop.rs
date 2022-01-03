use std::sync::Arc;

use twilight_model::application::interaction::ApplicationCommand;

use crate::{
    context::Context,
    error::BotResult,
    utils::{ApplicationCommandExt, MessageBuilder, SERVER_ID},
};

pub async fn stop(ctx: Arc<Context>, command: ApplicationCommand) -> BotResult<()> {
    info!("Clearing song queue and stopping current song...");
    if let Some(call) = ctx.songbird.get(SERVER_ID) {
        let call = call.lock().await;

        if call.queue().is_empty() {
            let builder = MessageBuilder::new().error("No song is currently playing!");
            return command.create_message(&ctx, builder).await;
        }

        call.queue().stop();

        let builder = MessageBuilder::new().embed("Stopped playing music!");
        return command.create_message(&ctx, builder).await;
    }

    Ok(())
}
