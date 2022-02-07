mod info;
mod leaderboard;
mod request;
mod request_list;
mod start;

use std::sync::Arc;

use twilight_model::application::{
    command::{ChoiceCommandOptionData, CommandOption, OptionsCommandOptionData},
    interaction::{
        application_command::{CommandData, CommandDataOption},
        ApplicationCommand,
    },
};

use crate::{context::Context, error::BotResult, utils::osu::get_osu_map_id};

#[command]
#[args = "OsuVSArgs"]
#[description = "Perform commands related to osuvs"]
#[options = "osuvs_options"]
pub struct OsuVS;

pub enum OsuVSArgs {
    Info,
    Leaderboard,
    RequestList,
    Request(Option<u32>),
    Start(Option<u32>),
}

impl OsuVSArgs {
    async fn parse_options(_: Arc<Context>, data: CommandData) -> BotResult<Self> {
        for option in data.options {
            if let CommandDataOption::SubCommand { name, options } = option {
                match name.as_str() {
                    "info" => return Ok(Self::Info),
                    "leaderboard" => return Ok(Self::Leaderboard),
                    "requestlist" => return Ok(Self::RequestList),
                    "request" => return Self::parse_request_options(options),
                    "start" => return Self::parse_start_options(options),
                    _ => (),
                }
            }
        }

        unreachable!();
    }

    fn parse_request_options(options: Vec<CommandDataOption>) -> BotResult<Self> {
        for option in options {
            if let CommandDataOption::String { name, value } = option {
                if name == "map" {
                    let map_id = get_osu_map_id(&value);
                    return Ok(Self::Request(map_id));
                }
            }
        }

        unreachable!()
    }

    fn parse_start_options(options: Vec<CommandDataOption>) -> BotResult<Self> {
        for option in options {
            if let CommandDataOption::String { name, value } = option {
                if name == "map" {
                    let map_id = get_osu_map_id(&value);
                    return Ok(Self::Start(map_id));
                }
            }
        }

        unreachable!()
    }
}

fn osuvs_options() -> Vec<CommandOption> {
    let info = OptionsCommandOptionData {
        description: "Get information about the current osuvs map".to_string(),
        name: "info".to_string(),
        options: vec![],
        required: false,
    };

    let leaderboard = OptionsCommandOptionData {
        description: "Get the leaderboard for the current osuvs map".to_string(),
        name: "leaderboard".to_string(),
        options: vec![],
        required: false,
    };

    let request_list = OptionsCommandOptionData {
        description: "Get the list of osuvs map requests. Only admins can use this".to_string(),
        name: "requestlist".to_string(),
        options: vec![],
        required: false,
    };

    let request_option = ChoiceCommandOptionData {
        choices: vec![],
        description: "Specify the map url (difficulty, not mapset)".to_string(),
        name: "map".to_string(),
        required: true,
    };

    let request = OptionsCommandOptionData {
        description: "Request a map for a future osuvs".to_string(),
        name: "request".to_string(),
        options: vec![CommandOption::String(request_option)],
        required: false,
    };

    let start_option = ChoiceCommandOptionData {
        choices: vec![],
        description: "Specify the map url (difficulty, not mapset)".to_string(),
        name: "map".to_string(),
        required: true,
    };

    let start = OptionsCommandOptionData {
        description: "Start or enqueue a map to the list of osuvs maps".to_string(),
        name: "start".to_string(),
        options: vec![CommandOption::String(start_option)],
        required: false,
    };

    vec![
        CommandOption::SubCommand(info),
        CommandOption::SubCommand(leaderboard),
        CommandOption::SubCommand(request_list),
        CommandOption::SubCommand(request),
        CommandOption::SubCommand(start),
    ]
}

async fn osuvs(ctx: Arc<Context>, command: ApplicationCommand, args: OsuVSArgs) -> BotResult<()> {
    match args {
        OsuVSArgs::Info => info::info(ctx, command).await,
        OsuVSArgs::Leaderboard => leaderboard::leaderboard(ctx, command).await,
        OsuVSArgs::RequestList => request_list::request_list(ctx, command).await,
        OsuVSArgs::Request(map_id) => request::request(ctx, command, map_id).await,
        OsuVSArgs::Start(map_id) => start::start(ctx, command, map_id).await,
    }
}
