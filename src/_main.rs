#[macro_use]
extern crate log;

use futures::StreamExt;
use songbird::{
    input::{Input, Restartable},
    tracks::{PlayMode, TrackHandle},
    Songbird,
};
use std::{collections::HashMap, env, error::Error, future::Future, sync::Arc};
use tokio::sync::RwLock;
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{Cluster, Event, Intents};
use twilight_http::Client as HttpClient;
use twilight_model::{channel::Message, gateway::payload::MessageCreate, id::GuildId, voice};
use twilight_standby::Standby;

type State = Arc<StateRef>;

#[derive(Debug)]
struct StateRef {
    cluster: Cluster,
    http: HttpClient,
    trackdata: RwLock<HashMap<GuildId, TrackHandle>>,
    songbird: Songbird,
    standby: Standby,
    cache: InMemoryCache,
}

fn spawn(
    fut: impl Future<Output = Result<(), Box<dyn Error + Send + Sync + 'static>>> + Send + 'static,
) {
    tokio::spawn(async move {
        if let Err(why) = fut.await {
            debug!("handler error: {:?}", why);
        }
    });
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    dotenv::dotenv().ok();
    env_logger::init();

    // Initialize the tracing subscriber.
    let (mut events, state) = {
        let token = env::var("DISCORD_TOKEN")?;

        let http = HttpClient::new(token.clone());
        let user_id = http.current_user().exec().await?.model().await?.id;

        let intents = Intents::GUILD_MESSAGES | Intents::GUILD_VOICE_STATES;
        let (cluster, events) = Cluster::new(token, intents).await?;
        cluster.up().await;

        let songbird = Songbird::twilight(cluster.clone(), user_id);
        let cache = InMemoryCache::builder()
            .resource_types(ResourceType::VOICE_STATE)
            .build();

        (
            events,
            Arc::new(StateRef {
                cluster,
                http,
                trackdata: Default::default(),
                songbird,
                standby: Standby::new(),
                cache,
            }),
        )
    };

    while let Some((_, event)) = events.next().await {
        state.standby.process(&event);
        state.songbird.process(&event).await;
        state.cache.update(&event);

        match event {
            Event::MessageCreate(msg) => {
                if msg.guild_id.is_none() || !msg.content.starts_with('!') {
                    continue;
                }

                match msg.content.splitn(2, ' ').next() {
                    Some("!join") => spawn(join(msg.0, Arc::clone(&state))),
                    Some("!leave") => spawn(leave(msg.0, Arc::clone(&state))),
                    Some("!pause") => spawn(pause(msg.0, Arc::clone(&state))),
                    Some("!play") => spawn(play(msg.0, Arc::clone(&state))),
                    Some("!seek") => spawn(seek(msg.0, Arc::clone(&state))),
                    Some("!stop") => spawn(stop(msg.0, Arc::clone(&state))),
                    Some("!volume") => spawn(volume(msg.0, Arc::clone(&state))),
                    _ => continue,
                }
            }
            _ => (),
        }
    }

    Ok(())
}

async fn join(msg: Message, state: State) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let author_id = msg.author.id;
    let guild_id = msg.guild_id.ok_or("Can't join a non-guild channel.")?;

    let channel_id = match state
        .cache
        .voice_state(author_id, guild_id)
        .and_then(|state| state.channel_id)
    {
        Some(state) => state,
        None => {
            state
                .http
                .create_message(msg.channel_id)
                .content("You aren't in a voice channel!")?
                .exec()
                .await?;
            return Ok(());
        }
    };

    let (_handle, success) = state.songbird.join(guild_id, channel_id).await;

    let content = match success {
        Ok(()) => format!("Joined <#{}>!", channel_id),
        Err(e) => format!("Failed to join <#{}>! Why: {:?}", channel_id, e),
    };

    state
        .http
        .create_message(msg.channel_id)
        .content(&content)?
        .exec()
        .await?;

    Ok(())
}

async fn leave(msg: Message, state: State) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    debug!(
        "leave command in channel {} by {}",
        msg.channel_id, msg.author.name
    );

    let guild_id = msg.guild_id.unwrap();

    state.songbird.leave(guild_id).await?;

    state
        .http
        .create_message(msg.channel_id)
        .content("Left the channel")?
        .exec()
        .await?;

    Ok(())
}

async fn play(msg: Message, state: State) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    debug!(
        "play command in channel {} by {}",
        msg.channel_id, msg.author.name
    );
    state
        .http
        .create_message(msg.channel_id)
        .content("What's the URL of the audio to play?")?
        .exec()
        .await?;

    let author_id = msg.author.id;
    let msg = state
        .standby
        .wait_for_message(msg.channel_id, move |new_msg: &MessageCreate| {
            new_msg.author.id == author_id
        })
        .await?;

    let guild_id = msg.guild_id.unwrap();

    match Restartable::ytdl_search(msg.content.clone(), false).await {
        Ok(song) => {
            let input = Input::from(song);

            let content = format!(
                "Playing **{:?}** by **{:?}**",
                input
                    .metadata
                    .track
                    .as_ref()
                    .unwrap_or(&"<UNKNOWN>".to_string()),
                input
                    .metadata
                    .artist
                    .as_ref()
                    .unwrap_or(&"<UNKNOWN>".to_string()),
            );

            state
                .http
                .create_message(msg.channel_id)
                .content(&content)?
                .exec()
                .await?;

            if let Some(call_lock) = state.songbird.get(guild_id) {
                let mut call = call_lock.lock().await;
                let handle = call.play_source(input);

                let mut store = state.trackdata.write().await;
                store.insert(guild_id, handle);
            }
        }
        Err(e) => {
            error!("{}", e);
            state
                .http
                .create_message(msg.channel_id)
                .content("Didn't find any results")?
                .exec()
                .await?;
        }
    }

    Ok(())
}

async fn pause(msg: Message, state: State) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    debug!(
        "pause command in channel {} by {}",
        msg.channel_id, msg.author.name
    );

    let guild_id = msg.guild_id.unwrap();

    let store = state.trackdata.read().await;

    let content = if let Some(handle) = store.get(&guild_id) {
        let info = handle.get_info().await?;

        let paused = match info.playing {
            PlayMode::Play => {
                let _success = handle.pause();
                false
            }
            _ => {
                let _success = handle.play();
                true
            }
        };

        let action = if paused { "Unpaused" } else { "Paused" };

        format!("{} the track", action)
    } else {
        format!("No track to (un)pause!")
    };

    state
        .http
        .create_message(msg.channel_id)
        .content(&content)?
        .exec()
        .await?;

    Ok(())
}

async fn seek(msg: Message, state: State) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    debug!(
        "seek command in channel {} by {}",
        msg.channel_id, msg.author.name
    );
    state
        .http
        .create_message(msg.channel_id)
        .content("Where in the track do you want to seek to (in seconds)?")?
        .exec()
        .await?;

    let author_id = msg.author.id;
    let msg = state
        .standby
        .wait_for_message(msg.channel_id, move |new_msg: &MessageCreate| {
            new_msg.author.id == author_id
        })
        .await?;
    let guild_id = msg.guild_id.unwrap();
    let position = msg.content.parse::<u64>()?;

    let store = state.trackdata.read().await;

    let content = if let Some(handle) = store.get(&guild_id) {
        if handle.is_seekable() {
            let _success = handle.seek_time(std::time::Duration::from_secs(position));
            format!("Seeked to {}s", position)
        } else {
            format!("Track is not compatible with seeking!")
        }
    } else {
        format!("No track to seek over!")
    };

    state
        .http
        .create_message(msg.channel_id)
        .content(&content)?
        .exec()
        .await?;

    Ok(())
}

async fn stop(msg: Message, state: State) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    debug!(
        "stop command in channel {} by {}",
        msg.channel_id, msg.author.name
    );

    let guild_id = msg.guild_id.unwrap();

    if let Some(call_lock) = state.songbird.get(guild_id) {
        let mut call = call_lock.lock().await;
        let _ = call.stop();
    }

    state
        .http
        .create_message(msg.channel_id)
        .content("Stopped the track")?
        .exec()
        .await?;

    Ok(())
}

async fn volume(msg: Message, state: State) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    debug!(
        "volume command in channel {} by {}",
        msg.channel_id, msg.author.name
    );
    state
        .http
        .create_message(msg.channel_id)
        .content("What's the volume you want to set (0.0-10.0, 1.0 being the default)?")?
        .exec()
        .await?;

    let author_id = msg.author.id;
    let msg = state
        .standby
        .wait_for_message(msg.channel_id, move |new_msg: &MessageCreate| {
            new_msg.author.id == author_id
        })
        .await?;
    let guild_id = msg.guild_id.unwrap();
    let volume = msg.content.parse::<f64>()?;

    if !volume.is_finite() || volume > 10.0 || volume < 0.0 {
        state
            .http
            .create_message(msg.channel_id)
            .content("Invalid volume!")?
            .exec()
            .await?;

        return Ok(());
    }

    let store = state.trackdata.read().await;

    let content = if let Some(handle) = store.get(&guild_id) {
        let _success = handle.set_volume(volume as f32);
        format!("Set the volume to {}", volume)
    } else {
        format!("No track to change volume!")
    };

    state
        .http
        .create_message(msg.channel_id)
        .content(&content)?
        .exec()
        .await?;

    Ok(())
}
