mod message;
mod music;
mod osu;
mod utils;

use std::sync::Arc;

use twilight_model::application::{command::Command, interaction::ApplicationCommand};
use utils::Ping;

use crate::{
    context::Context,
    error::{BotResult, Error},
    utils::ApplicationCommandExt,
};

const NUM_COMMANDS: usize = 1;

pub fn twilight_commands() -> [Command; NUM_COMMANDS] {
    [Ping::define()]
}

fn log_slash(ctx: &Context, command: &ApplicationCommand, cmd_name: &str) {
    let username = command.username().unwrap_or("<unknown user>");
    let mut location = String::with_capacity(31);

    match command.guild_id.and_then(|id| ctx.cache.guild(id)) {
        Some(guild) => {
            location.push_str(guild.name.as_str());
            location.push(':');

            match ctx.cache.guild_channel(command.channel_id) {
                Some(channel) => location.push_str(channel.name()),
                None => location.push_str("<uncached channel>"),
            }
        }
        None => location.push_str("Private"),
    }

    info!("[{}] {}: /{}", location, username, cmd_name);
}

pub async fn handle_interaction(ctx: Arc<Context>, command: ApplicationCommand) -> BotResult<()> {
    let name = command.data.name.as_str();
    log_slash(&ctx, &command, name);
    ctx.stats.increment_slash_command(name);

    match name {
        Ping::NAME => Ping::run(ctx, command).await,
        _ => Err(Error::UnknownInteraction {
            command: Box::new(command),
        }),
    }
}
