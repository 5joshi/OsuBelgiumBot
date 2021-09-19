use std::env::VarError;
use std::error::Error as StdError;
use std::fmt;
use twilight_gateway::cluster::ClusterStartError;
use twilight_http::response::DeserializeBodyError;
use twilight_http::Error as TwilightHttp;

#[derive(Debug)]
pub enum Error {
    TwilightHttp { src: TwilightHttp },
    DeserializeBody { src: DeserializeBodyError },
    ClusterStart { src: ClusterStartError },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::TwilightHttp { .. } => f.write_str("Error while using Twilight HTTP."),
            Error::DeserializeBody { .. } => f.write_str("Failed to deserialize Discord object."),
            Error::ClusterStart { .. } => f.write_str("Failed to start cluster."),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::TwilightHttp { src } => Some(src),
            Error::DeserializeBody { src } => Some(src),
            Error::ClusterStart { src } => Some(src),
        }
    }
}

impl From<TwilightHttp> for Error {
    fn from(src: twilight_http::Error) -> Self {
        Self::TwilightHttp { src }
    }
}

impl From<DeserializeBodyError> for Error {
    fn from(src: DeserializeBodyError) -> Self {
        Self::DeserializeBody { src }
    }
}

impl From<ClusterStartError> for Error {
    fn from(src: ClusterStartError) -> Self {
        Self::ClusterStart { src }
    }
}
