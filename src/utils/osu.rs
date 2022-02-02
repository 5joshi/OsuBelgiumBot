use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use bytes::Bytes;
use regex::Regex;
use rosu_v2::prelude::Beatmap;
use tokio::{fs::File, io::AsyncWriteExt, time::sleep};

use crate::error::MapDownloadError;

use super::OSU_BASE;

lazy_static! {
    static ref OSU_URL_MAP_NEW_MATCHER: Regex = Regex::new(
        r"https://osu.ppy.sh/beatmapsets/(\d+)(?:(?:#(?:osu|mania|taiko|fruits)|<#\d+>)/(\d+))?"
    )
    .unwrap();
    static ref OSU_URL_MAP_OLD_MATCHER: Regex =
        Regex::new(r"https://osu.ppy.sh/b(?:eatmaps)?/(\d+)").unwrap();
}

pub fn get_osu_map_id(msg: &str) -> Option<u32> {
    if let Ok(id) = msg.parse::<u32>() {
        return Some(id);
    }

    if !msg.contains(OSU_BASE) {
        return None;
    }

    let matcher = if let Some(c) = OSU_URL_MAP_OLD_MATCHER.captures(msg) {
        c.get(1)
    } else {
        OSU_URL_MAP_NEW_MATCHER.captures(msg).and_then(|c| c.get(2))
    };

    matcher.and_then(|c| c.as_str().parse::<u32>().ok())
}

pub fn map_to_string(map: &Beatmap) -> String {
    let mapset = map.mapset.as_ref().unwrap();
    format!("{} - {} [{}]", mapset.artist, mapset.title, map.version)
}

pub async fn prepare_beatmap_file(map_id: u32) -> Result<String, MapDownloadError> {
    let mut map_path = PathBuf::new();
    map_path.push(
        std::env::var("BEATMAPS_FOLDER").expect("Missing environment variable (BEATMAPS_FOLDER)."),
    );
    map_path.push(format!("{map_id}.osu"));

    if !map_path.exists() {
        let content = request_beatmap_file(map_id).await?;
        let mut file = File::create(&map_path).await?;
        file.write_all(&content).await?;
        info!("Downloaded {map_id}.osu successfully");
    }

    let map_path = map_path
        .into_os_string()
        .into_string()
        .expect("map_path OsString is no valid String");

    Ok(map_path)
}

async fn request_beatmap_file(map_id: u32) -> Result<Bytes, MapDownloadError> {
    let url = format!("{OSU_BASE}osu/{map_id}");
    let mut content = reqwest::get(&url).await?.bytes().await?;

    if content.len() >= 6 && &content.slice(0..6)[..] != b"<html>" {
        return Ok(content);
    }

    // 1s - 2s - 4s - 8s - 10s - ...
    let backoff = ExponentialBackoff::new(2).factor(500).max_delay(10_000);

    for (duration, i) in backoff.take(10).zip(1..) {
        debug!("Request beatmap retry attempt #{i} | Backoff {duration:?}");
        sleep(duration).await;

        content = reqwest::get(&url).await?.bytes().await?;

        if content.len() >= 6 && &content.slice(0..6)[..] != b"<html>" {
            return Ok(content);
        }
    }

    (content.len() >= 6 && &content.slice(0..6)[..] != b"<html>")
        .then(|| content)
        .ok_or(MapDownloadError::RetryLimit(map_id))
}

#[derive(Debug, Clone)]
pub struct ExponentialBackoff {
    current: Duration,
    base: u32,
    factor: u32,
    max_delay: Option<Duration>,
}

impl ExponentialBackoff {
    pub fn new(base: u32) -> Self {
        ExponentialBackoff {
            current: Duration::from_millis(base as u64),
            base,
            factor: 1,
            max_delay: None,
        }
    }

    pub fn factor(mut self, factor: u32) -> Self {
        self.factor = factor;

        self
    }

    pub fn max_delay(mut self, max_delay: u64) -> Self {
        self.max_delay.replace(Duration::from_millis(max_delay));

        self
    }
}

impl Iterator for ExponentialBackoff {
    type Item = Duration;

    fn next(&mut self) -> Option<Duration> {
        let duration = self.current * self.factor;

        if let Some(max_delay) = self.max_delay.filter(|&max_delay| duration > max_delay) {
            return Some(max_delay);
        }

        self.current *= self.base;

        Some(duration)
    }
}
