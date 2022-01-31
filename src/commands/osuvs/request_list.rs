use std::sync::Arc;

use twilight_model::application::interaction::ApplicationCommand;

use crate::{context::Context, error::BotResult};

pub async fn request_list(ctx: Arc<Context>, command: ApplicationCommand) -> BotResult<()> {
    todo!()
}
