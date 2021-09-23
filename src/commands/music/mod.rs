mod play;
mod skip;

use std::sync::Arc;

use twilight_model::application::{
    command::{ChoiceCommandOptionData, CommandOption, OptionsCommandOptionData},
    interaction::{
        application_command::{CommandData, CommandDataOption},
        ApplicationCommand,
    },
};

use crate::{context::Context, error::BotResult};

#[command]
#[args = "MusicArgs"]
#[description = "Manipulate music"]
#[options = "music_options"]
pub struct Music;

pub enum MusicArgs {
    Play(String),
    Skip(usize),
}

impl MusicArgs {
    async fn parse_options(_: Arc<Context>, data: CommandData) -> BotResult<Self> {
        for option in data.options {
            if let CommandDataOption::SubCommand { name, options } = option {
                match name.as_str() {
                    "play" => return Self::parse_play_options(options),
                    "skip" => return Self::parse_skip_options(options),
                    _ => (),
                }
            }
        }

        unreachable!();
    }

    fn parse_play_options(options: Vec<CommandDataOption>) -> BotResult<Self> {
        for option in options {
            if let CommandDataOption::String { name, value } = option {
                if name == "song" {
                    return Ok(Self::Play(value));
                }
            }
        }

        unreachable!()
    }

    fn parse_skip_options(options: Vec<CommandDataOption>) -> BotResult<Self> {
        for option in options {
            if let CommandDataOption::Integer { name, value } = option {
                if name == "amount" {
                    return Ok(Self::Skip(value.max(0) as usize));
                }
            }
        }

        Ok(Self::Skip(1))
    }
}

fn music_options() -> Vec<CommandOption> {
    let play_option = ChoiceCommandOptionData {
        choices: vec![],
        description: "Specify a song name or youtube url".to_string(),
        name: "song".to_string(),
        required: true,
    };

    let play = OptionsCommandOptionData {
        description: "Play a given song".to_string(),
        name: "play".to_string(),
        options: vec![CommandOption::String(play_option)],
        required: false,
    };

    let skip_option = ChoiceCommandOptionData {
        choices: vec![],
        description: "Specify a number of songs to skip, skips one by default".to_string(),
        name: "amount".to_string(),
        required: false,
    };

    let skip = OptionsCommandOptionData {
        description: "Skip a number of songs in the queue".to_string(),
        name: "skip".to_string(),
        options: vec![CommandOption::Integer(skip_option)],
        required: false,
    };

    vec![
        CommandOption::SubCommand(play),
        CommandOption::SubCommand(skip),
    ]
}

async fn music(ctx: Arc<Context>, command: ApplicationCommand, args: MusicArgs) -> BotResult<()> {
    match args {
        MusicArgs::Play(song) => play::play(ctx, command, song).await,
        MusicArgs::Skip(amount) => skip::skip(ctx, command, amount).await,
    }
}
