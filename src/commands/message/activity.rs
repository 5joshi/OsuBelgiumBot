use std::sync::Arc;

use rand::Rng;
use twilight_model::{
    application::{
        callback::{CallbackData, InteractionResponse},
        command::{BaseCommandOptionData, CommandOption},
        interaction::{
            application_command::{CommandData, CommandDataOption, InteractionChannel},
            ApplicationCommand,
        },
    },
    channel::ChannelType,
    id::ChannelId,
};

use crate::{context::Context, error::BotResult};

#[command]
#[args = "ActivityArgs"]
#[description = "Get the server activity for the last month, week, day and hour"]
#[options = "activity_options"]
pub struct Activity;

pub struct ActivityArgs {
    channel: Option<InteractionChannel>,
}

impl ActivityArgs {
    async fn parse_options(ctx: Arc<Context>, data: CommandData) -> BotResult<Self> {
        let channel = data.resolved.and_then(|mut data| data.channels.pop());
        Ok(Self { channel })
    }
}

fn activity_options() -> Vec<CommandOption> {
    let option_data = BaseCommandOptionData {
        description: "Specify an optional channel to check the activity for".to_string(),
        name: "channel".to_string(),
        required: false,
    };

    vec![CommandOption::Channel(option_data)]
}

async fn activity(
    ctx: Arc<Context>,
    command: ApplicationCommand,
    args: ActivityArgs,
) -> BotResult<()> {
    if args
        .channel
        .filter(|channel| channel.kind != ChannelType::GuildText)
        .is_some()
    {
        let response = InteractionResponse::ChannelMessageWithSource(CallbackData {
            allowed_mentions: None,
            components: None,
            content: Some(format!("Please specify a regular text channel!")),
            embeds: vec![],
            flags: None,
            tts: None,
        });

        ctx.http
            .interaction_callback(command.id, &command.token, &response)
            .exec()
            .await?;
    }

    let response = InteractionResponse::ChannelMessageWithSource(CallbackData {
        allowed_mentions: None,
        components: None,
        content: Some(format!("You rolled! :game_die:")),
        embeds: vec![],
        flags: None,
        tts: None,
    });

    ctx.http
        .interaction_callback(command.id, &command.token, &response)
        .exec()
        .await?;

    Ok(())
}

#[derive(Default)]
pub struct MessageActivity {
    pub month: usize,
    pub week: usize,
    pub day: usize,
    pub hour: usize,
}
