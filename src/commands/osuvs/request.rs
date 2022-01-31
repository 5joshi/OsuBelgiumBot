use std::sync::Arc;

use twilight_model::application::interaction::ApplicationCommand;

use crate::{context::Context, error::BotResult};

pub async fn request(
    ctx: Arc<Context>,
    command: ApplicationCommand,
    map_id: Result<u32, &str>,
) -> BotResult<()> {
    todo!()
}
