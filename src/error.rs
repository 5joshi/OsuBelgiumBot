use irc::error::Error as IrcError;
use rosu_v2::prelude::OsuError;
use std::env::VarError;
use std::error::Error as StdError;
use std::fmt;
use twilight_gateway::cluster::{ClusterCommandError, ClusterStartError};
use twilight_http::response::DeserializeBodyError;
use twilight_http::Error as TwilightHttpError;

pub type BotResult<T> = Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ClusterCommand { src: ClusterCommandError },
    ClusterStart { src: ClusterStartError },
    DeserializeBody { src: DeserializeBodyError },
    Irc { src: IrcError },
    Osu { src: OsuError },
    TwilightHttp { src: TwilightHttpError },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ClusterCommand { .. } => f.write_str("Error occurred on cluster request."),
            Error::ClusterStart { .. } => f.write_str("Failed to start cluster."),
            Error::DeserializeBody { .. } => f.write_str("Failed to deserialize Discord object."),
            Error::Irc { .. } => f.write_str("Failed to communicate with osu! IRC."),
            Error::Osu { .. } => f.write_str("Failed to communicate with osu! API."),
            Error::TwilightHttp { .. } => f.write_str("Error while using Twilight HTTP."),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::ClusterCommand { src } => Some(src),
            Error::ClusterStart { src } => Some(src),
            Error::DeserializeBody { src } => Some(src),
            Error::Irc { src } => Some(src),
            Error::Osu { src } => Some(src),
            Error::TwilightHttp { src } => Some(src),
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

impl From<DeserializeBodyError> for Error {
    fn from(src: DeserializeBodyError) -> Self {
        Self::DeserializeBody { src }
    }
}

impl From<IrcError> for Error {
    fn from(src: IrcError) -> Self {
        Self::Irc { src }
    }
}

impl From<OsuError> for Error {
    fn from(src: OsuError) -> Self {
        Self::Osu { src }
    }
}

impl From<TwilightHttpError> for Error {
    fn from(src: TwilightHttpError) -> Self {
        Self::TwilightHttp { src }
    }
}
