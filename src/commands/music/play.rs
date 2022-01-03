use std::sync::Arc;

use songbird::{
    input::{Input, Restartable},
    Event, EventContext, EventHandler, TrackEvent,
};
use twilight_model::{
    application::interaction::ApplicationCommand,
    gateway::presence::{ActivityType, Status},
};

use crate::{
    context::Context,
    error::BotResult,
    utils::{matcher, ApplicationCommandExt, EmbedBuilder, MessageBuilder},
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

    if let Err(success) = success {
        let builder = MessageBuilder::new().error("Failed to join voice channel! Blame Joshi :c");
        let _ = command.update_message(&ctx, builder).await;
        return Err(success.into());
    }

    info!(
        "Joined channel {} after play command by {}",
        if let Some(channel) = ctx.cache.guild_channel(channel_id) {
            channel.name().to_owned()
        } else {
            channel_id.to_string()
        },
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

            if let Some(call_lock) = ctx.songbird.get(guild_id) {
                let mut call = call_lock.lock().await;
                let empty = call.queue().is_empty();

                let mut metadata_str = match (
                    &input.metadata.track,
                    &input.metadata.artist,
                    &input.metadata.title,
                ) {
                    (Some(track), Some(artist), _) => format!("**{} - {}**", artist, track),
                    (.., Some(title)) => format!("**{}**", title),
                    _ => "**UNKNOWN**".to_string(),
                };

                if let Some(url) = &input.metadata.source_url {
                    metadata_str = format!("[{}]({})", metadata_str, url);
                }

                let content = format!(
                    "{}{}{}",
                    if empty { "Started playing " } else { "Added " },
                    metadata_str,
                    if empty { "" } else { " to the queue" },
                );

                info!("{}", content);

                let mut builder = EmbedBuilder::new().description(content);
                if let Some(ref thumbnail) = input.metadata.thumbnail {
                    builder = builder.image(thumbnail);
                }

                call.enqueue_source(input);
                call.queue().modify_queue(|q| {
                    q.back().map(|q| {
                        q.add_event(Event::Track(TrackEvent::Play), TrackStart(Arc::clone(&ctx)))
                    })
                });
                command.update_message(&ctx, builder).await?;

                // ctx.trackdata.write().replace(handle);
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

struct TrackStart(Arc<Context>);

#[async_trait]
impl EventHandler for TrackStart {
    async fn act(&self, ctx: &songbird::EventContext<'_>) -> Option<songbird::Event> {
        if let EventContext::Track(&[(state, track)]) = ctx {
            info!("Changing activity");

            let metadata = track.metadata();

            let mut metadata_str = match (&metadata.track, &metadata.artist, &metadata.title) {
                (Some(track), Some(artist), _) => format!("🎵 {} - {} 🎵", artist, track),
                (.., Some(title)) => format!("🎵 {} 🎵", title),
                _ => "🎵 UNKNOWN 🎵".to_string(),
            };
            let result = self
                .0
                .set_shard_activity(0, Status::Online, ActivityType::Playing, metadata_str)
                .await;

            if let Err(e) = result {
                unwind_error!(warn, e, "Failed to set song activity: {}")
            }
        }
        None
    }
}
