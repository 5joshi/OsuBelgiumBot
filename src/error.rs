use irc::error::Error as IrcError;
use rosu_v2::prelude::OsuError;
use sqlx::migrate::MigrateError;
use sqlx::Error as SqlError;
use std::error::Error as StdError;
use std::fmt;
use twilight_gateway::cluster::{ClusterCommandError, ClusterStartError};
use twilight_http::request::application::InteractionError;
use twilight_http::request::prelude::create_message::CreateMessageError;
use twilight_http::response::DeserializeBodyError;
use twilight_http::Error as TwilightHttpError;
use twilight_model::application::interaction::ApplicationCommand;

pub type BotResult<T> = Result<T, Error>;

#[rustfmt::skip]
#[derive(Debug)]
pub enum Error {
    ClusterCommand { src: ClusterCommandError },
    ClusterStart { src: ClusterStartError },
    CreateMessage { src: CreateMessageError },
    DeserializeBody { src: DeserializeBodyError },
    Interaction { src: InteractionError },
    Irc { src: IrcError },
    Migration { src: MigrateError },
    MissingSlashAuthor,
    Osu { src: OsuError },
    Sql { src: SqlError },
    TwilightHttp { src: TwilightHttpError },
    UnknownInteraction { command: Box<ApplicationCommand> },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ClusterCommand { .. } => f.write_str("Error occurred on cluster request."),
            Error::ClusterStart { .. } => f.write_str("Failed to start cluster."),
            Error::CreateMessage { .. } => f.write_str("Failed to create message."),
            Error::DeserializeBody { .. } => f.write_str("Failed to deserialize Discord object."),
            Error::Interaction { .. } => f.write_str("Failed to interact with Discord."),
            Error::Irc { .. } => f.write_str("Failed to communicate with osu! IRC."),
            Error::Migration { .. } => f.write_str("Failed to migrate database."),
            Error::MissingSlashAuthor => f.write_str("Slash author was not found."),
            Error::Osu { .. } => f.write_str("Failed to communicate with osu! API."),
            Error::Sql { .. } => f.write_str("Error caused by database."),
            Error::TwilightHttp { .. } => f.write_str("Error while using Twilight HTTP."),
            Error::UnknownInteraction { command } => {
                write!(
                    f,
                    "Received unknown interaction ({}): {:#?}",
                    command.data.name, command
                )
            }
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::ClusterCommand { src } => Some(src),
            Error::ClusterStart { src } => Some(src),
            Error::CreateMessage { src } => Some(src),
            Error::DeserializeBody { src } => Some(src),
            Error::Interaction { src } => Some(src),
            Error::Irc { src } => Some(src),
            Error::Osu { src } => Some(src),
            Error::Migration { src } => Some(src),
            Error::MissingSlashAuthor => None,
            Error::Sql { src } => Some(src),
            Error::TwilightHttp { src } => Some(src),
            Error::UnknownInteraction { .. } => None,
        }
    }
}

impl From<ClusterCommandError> for Error {
    fn from(src: ClusterCommandError) -> Self {
        Self::ClusterCommand { src }
    }
}

impl From<ClusterStartError> for Error {
    fn from(src: ClusterStartError) -> Self {
        Self::ClusterStart { src }
    }
}

impl From<CreateMessageError> for Error {
    fn from(src: CreateMessageError) -> Self {
        Self::CreateMessage { src }
    }
}

impl From<DeserializeBodyError> for Error {
    fn from(src: DeserializeBodyError) -> Self {
        Self::DeserializeBody { src }
    }
}

impl From<InteractionError> for Error {
    fn from(src: InteractionError) -> Self {
        Self::Interaction { src }
    }
}

impl From<IrcError> for Error {
    fn from(src: IrcError) -> Self {
        Self::Irc { src }
    }
}

impl From<MigrateError> for Error {
    fn from(src: MigrateError) -> Self {
        Self::Migration { src }
    }
}

impl From<OsuError> for Error {
    fn from(src: OsuError) -> Self {
        Self::Osu { src }
    }
}

impl From<SqlError> for Error {
    fn from(src: SqlError) -> Self {
        Self::Sql { src }
    }
}

impl From<TwilightHttpError> for Error {
    fn from(src: TwilightHttpError) -> Self {
        Self::TwilightHttp { src }
    }
}
