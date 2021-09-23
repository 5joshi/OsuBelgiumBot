use std::sync::Arc;

use twilight_model::application::interaction::ApplicationCommand;

use crate::{context::Context, error::BotResult};

pub async fn skip(ctx: Arc<Context>, command: ApplicationCommand, amount: usize) -> BotResult<()> {
    todo!()
}
