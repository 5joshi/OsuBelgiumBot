macro_rules! unwind_error {
    ($log:ident, $err:ident, $($arg:tt)+) => {
        {
            $log!($($arg)+, $err);
            let mut err: &dyn ::std::error::Error = &$err;

            while let Some(source) = err.source() {
                $log!("  - caused by: {}", source);
                err = source;
            }
        }
    };
}

mod commands;
mod context;
mod database;
mod error;
mod utils;

use context::Context;
use database::Database;
use error::Error;

use songbird::Songbird;
use std::env;
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{Cluster, Intents};
use twilight_http::Client as HttpClient;

#[macro_use]
extern crate log;

fn main() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed building the Runtime");
    if let Err(e) = runtime.block_on(async_main()) {
        unwind_error!(error, e, "Critical Error in main: {}")
    };
}

async fn async_main() -> Result<(), Error> {
    dotenv::dotenv().ok();
    env_logger::init();

    // Initialize the tracing subscriber.
    let token = env::var("DISCORD_TOKEN").expect("Missing environment variable (DISCORD_TOKEN)");

    let http = HttpClient::new(token.clone());
    let user_id = http.current_user().exec().await?.model().await?.id;

    let intents = Intents::GUILD_MESSAGES | Intents::GUILD_VOICE_STATES;
    let (cluster, events) = Cluster::new(token, intents).await?;
    cluster.up().await;

    let songbird = Songbird::twilight(cluster.clone(), user_id);
    let cache = InMemoryCache::builder()
        .resource_types(ResourceType::VOICE_STATE)
        .build();
    Ok(())
}
