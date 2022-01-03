use std::sync::Arc;

use twilight_model::application::interaction::ApplicationCommand;

use crate::{
    context::Context,
    error::BotResult,
    utils::{ApplicationCommandExt, MessageBuilder, SERVER_ID},
};

pub async fn volume(ctx: Arc<Context>, command: ApplicationCommand, volume: f32) -> BotResult<()> {
    info!("Setting song volume to {}...", volume);
    if let Some(call) = ctx.songbird.get(SERVER_ID) {
        let call = call.lock().await;

        let handle = match call.queue().current() {
            Some(handle) => handle,
            None => {
                let builder = MessageBuilder::new().error("No song is currently playing!");
                return command.create_message(&ctx, builder).await;
            }
        };

        handle.set_volume(volume);

        let content = format!("Changed volume to {}!", volume);
        let builder = MessageBuilder::new().embed(content);
        return command.create_message(&ctx, builder).await;
    }

    Ok(())
}
