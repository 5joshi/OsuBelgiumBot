use std::sync::Arc;

use songbird::input::{Input, Restartable};
use twilight_model::application::interaction::ApplicationCommand;

use crate::{
    context::Context,
    error::BotResult,
    utils::{matcher, ApplicationCommandExt, MessageBuilder},
};

pub async fn play(ctx: Arc<Context>, command: ApplicationCommand, song: String) -> BotResult<()> {
    command.start_thinking(&ctx).await?;

    let author_id = command.user_id()?;
    let guild_id = command.guild_id.expect("Missing Guild ID for play command");

    let channel_id = match ctx
        .cache
        .voice_state(author_id, guild_id)
        .and_then(|state| state.channel_id)
    {
        Some(id) => id,
        None => {
            let builder = MessageBuilder::new().error("You aren't in a voice channel!");
            return command.update_message(&ctx, builder).await;
        }
    };

    let (_handle, success) = ctx.songbird.join(guild_id, channel_id).await;

    info!("{:?}", _handle.lock().await.current_connection());
    if let Err(success) = success {
        let builder = MessageBuilder::new().error("Failed to join voice channel! Blame Joshi :c");
        let _ = command.update_message(&ctx, builder).await;
        return Err(success.into());
    }

    info!(
        "Joined channel {} after play command by {}",
        channel_id,
        command.username()?
    );

    let id = matcher::get_youtube_id(&song);
    let yt_search = if let Some(_) = id {
        song
    } else {
        format!("ytsearch1:{}", song)
    };

    match Restartable::ytdl_search(&yt_search, false).await {
        Ok(song) => {
            let input = Input::from(song);

            let content = format!(
                "Playing **{:?}** by **{:?}**",
                input.metadata.track.as_deref().unwrap_or("<UNKNOWN>"),
                input.metadata.artist.as_deref().unwrap_or("<UNKNOWN>"),
            );

            info!("{}", content);
            let builder = MessageBuilder::new().embed(content);
            command.update_message(&ctx, builder).await?;

            if let Some(call_lock) = ctx.songbird.get(guild_id) {
                let mut call = call_lock.lock().await;
                let handle = call.play_source(input);

                ctx.trackdata.write().replace(handle);
            } else {
                error!("cringe digga");
            }
        }
        Err(e) => {
            unwind_error!(
                error,
                e,
                "No youtube search results found for query {}: {}",
                yt_search
            );
            let builder = MessageBuilder::new().embed("Didn't find any results");
            command.update_message(&ctx, builder).await?;
        }
    }
    Ok(())
}
