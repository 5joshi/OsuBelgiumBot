use std::sync::Arc;

use twilight_model::application::{
    callback::{CallbackData, InteractionResponse},
    command::Command,
    interaction::ApplicationCommand,
};

use crate::{commands::SlashCommand, context::Context, error::BotResult};

pub struct Ping(pub ApplicationCommand);

impl Ping {
    pub async fn run(self, ctx: Arc<Context>) -> BotResult<()> {
        let response = InteractionResponse::ChannelMessageWithSource(CallbackData {
            allowed_mentions: None,
            components: None,
            content: Some("BIG CHUNGUS".to_string()),
            embeds: vec![],
            flags: None,
            tts: None,
        });

        ctx.http
            .interaction_callback(self.0.id, &self.0.token, &response)
            .exec()
            .await?;

        Ok(())
    }
}

impl SlashCommand for Ping {
    const NAME: &'static str = "ping";

    fn define() -> Command {
        Command {
            application_id: None,
            guild_id: None,
            name: "ping".to_string(),
            default_permission: None,
            description: "Play ping pong with the bot".to_string(),
            id: None,
            options: vec![],
        }
    }
}
