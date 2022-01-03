use std::sync::Arc;

use twilight_model::application::interaction::ApplicationCommand;

use crate::{
    context::Context,
    error::BotResult,
    utils::{ApplicationCommandExt, MessageBuilder, SERVER_ID},
};

pub async fn skip(ctx: Arc<Context>, command: ApplicationCommand, amount: usize) -> BotResult<()> {
    info!("Skipping {} song(s) in song queue...", amount);
    if amount == 0 {
        let builder = MessageBuilder::new().error("Stop trying to break the bot.");
        return command.create_message(&ctx, builder).await;
    }

    if let Some(call) = ctx.songbird.get(SERVER_ID) {
        let call = call.lock().await;

        if call.queue().is_empty() {
            let builder = MessageBuilder::new().error("No song is currently playing!");
            return command.create_message(&ctx, builder).await;
        }

        for _ in 0..amount.min(call.queue().len()) {
            let success = call.queue().skip();

            if let Err(e) = success {
                let builder =
                    MessageBuilder::new().error("Failed to skip all of the songs! Blame Joshi :c");
                let _ = command.create_message(&ctx, builder).await;
                return Err(e.into());
            }
        }

        let content = format!(
            "Skipped {} song{}!",
            amount,
            if amount != 1 { "s" } else { "" }
        );
        let builder = MessageBuilder::new().embed(content);
        let _ = command.create_message(&ctx, builder).await;
    }

    Ok(())
}
