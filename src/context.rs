use crate::Database;

use irc::client::prelude::Client as IrcClient;
use parking_lot::RwLock;
use rosu_v2::Osu as OsuClient;
use songbird::{tracks::TrackHandle, Songbird};
use twilight_cache_inmemory::InMemoryCache;
use twilight_gateway::Cluster;
use twilight_http::Client as HttpClient;
use twilight_standby::Standby;

pub struct Context {
    pub database: Database,
    pub osu: OsuClient,
    pub irc: IrcClient,
    pub cluster: Cluster,
    pub http: HttpClient,
    pub trackdata: RwLock<TrackHandle>,
    pub songbird: Songbird,
    pub standby: Standby,
    pub cache: InMemoryCache,
}
