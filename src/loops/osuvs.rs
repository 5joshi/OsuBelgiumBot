use chrono::{DateTime, Duration as ChronoDuration, Utc};
use futures::{
    future::{try_join_all, FutureExt},
    stream::{FuturesUnordered, StreamExt},
};
use itertools::Itertools;
use rosu_pp::{osu::OsuPerformanceAttributes, Beatmap, OsuPP};
use rosu_v2::prelude::{GameMode, GameMods, Grade, Score};
use std::fmt::Write;
use std::{collections::HashMap, sync::Arc};
use tokio::time::{interval, Duration};

use crate::{
    context::Context,
    error::BotResult,
    utils::{
        datetime::sec_to_minsec,
        numbers::{round, with_comma_uint},
        osu::{map_to_string, prepare_beatmap_file},
        ApplicationCommandExt, Author, EmbedBuilder, Footer, EMOTE_MEDALS, EMOTE_RANKS,
        OSUVS_CHANNEL, OSUVS_DATE_FORMAT, OSU_BASE,
    },
};

const OSUVS_TRACK_INTERVAL: u64 = 300;

pub async fn osu_tracking(ctx: Arc<Context>) {
    let mut interval = interval(Duration::from_secs(OSUVS_TRACK_INTERVAL));
    interval.tick().await;

    let mut user_ids = HashMap::with_capacity(ctx.irc.targets.len());

    loop {
        interval.tick().await;

        let (map_id, start, end) = match ctx.database.get_curr_osuvs_map().await {
            Some(tuple) => tuple,
            None => continue,
        };

        let now = Utc::now();
        if now - start < ChronoDuration::seconds(OSUVS_TRACK_INTERVAL as i64) {
            if let Err(why) = map_start(&ctx, map_id, end).await {
                unwind_error!(error, why, "Error while announcing end of osuvs map: {}");
            }
        }

        let mut users = Vec::new();

        for name in ctx.irc.online.iter() {
            if !user_ids.contains_key(name.as_str()) {
                match ctx.osu.user(name.as_str()).await {
                    Ok(user) => {
                        user_ids.insert(name.to_owned(), user.user_id);
                    }
                    Err(why) => {
                        unwind_error!(warn, why, "Failed to add user id`: {}");
                        continue;
                    }
                }
            }

            users.push(user_ids[name.as_str()]);
        }

        debug!("[Track] {} users: {:?}", users.len(), users);
        loop_iteration(&ctx, map_id, &users, start).await;

        if end - now < ChronoDuration::seconds(OSUVS_TRACK_INTERVAL as i64) {
            if let Err(why) = map_end(&ctx, map_id).await {
                unwind_error!(error, why, "Error while announcing end of osuvs map: {}");
            }
        }
    }
}

async fn loop_iteration(ctx: &Context, map_id: u32, users: &[u32], start: DateTime<Utc>) {
    // Request the last 50 recents scores for all online tracked users
    let scores: HashMap<u32, Vec<Score>> = {
        users
            .iter()
            .map(|&user_id| {
                ctx.osu
                    .user_scores(user_id)
                    .recent()
                    .mode(GameMode::STD)
                    .limit(50)
                    .map(move |res| (user_id, res))
            })
            .collect::<FuturesUnordered<_>>()
            .filter_map(|(user_id, res)| async move {
                match res {
                    Ok(scores) => Some((user_id, scores)),
                    Err(why) => {
                        unwind_error!(warn, why, "Error while requesting tracked user: {}");

                        None
                    }
                }
            })
            .collect()
            .await
    };

    // Map each user to a vec containing the best score
    // on the osuvs map for each played mod
    let recent_best: HashMap<_, _> = scores
        .into_iter()
        .filter_map(|(_, scores)| {
            let user_id = scores.first().map(|s| s.user_id)?;

            let scores: Vec<_> = scores
                .into_iter()
                .filter(|s| s.map.as_ref().unwrap().map_id == map_id)
                .filter(|s| s.grade(None) != Grade::F)
                .filter(|s| s.created_at >= start)
                .filter(|s| !s.mods.contains(GameMods::ScoreV2))
                .map(|s| (s.mods.bits(), s))
                .sorted_by(|(m1, _), (m2, _)| m1.cmp(m2))
                .group_by(|(mods, _)| *mods)
                .into_iter()
                .flat_map(|(_, group)| {
                    group
                        .sorted_by(|(_, s1), (_, s2)| s2.score.cmp(&s1.score))
                        .next()
                })
                .collect();

            (!scores.is_empty()).then(|| (user_id, scores))
        })
        .collect();

    // If no one played the map, skip loop iteration
    if recent_best.is_empty() {
        return;
    }

    let total_best: HashMap<u32, Score> = match ctx.database.get_osuvs_highscores(200).await {
        Ok(highscores) => highscores.into_iter().collect(),
        Err(why) => {
            unwind_error!(error, why, "Error while getting OsuVS highscores: {}");

            return;
        }
    };

    if !total_best.is_empty() {
        for (user, score) in total_best {
            if let Err(why) = ctx
                .database
                .insert_osuvs_highscores(map_id, user, score)
                .await
            {
                unwind_error!(error, why, "Error while inserting new OsuVS scores: {}");
            }
        }
    }
}

async fn map_start(ctx: &Context, map_id: u32, end: DateTime<Utc>) -> BotResult<()> {
    info!("Starting osuvs map, sending message...");
    let map = ctx.osu.beatmap().map_id(map_id).await?;
    let title = map_to_string(&map);
    let url = format!("{}b/{}", OSU_BASE, map_id);
    let author = Author::new("A new OsuVS has started!");
    let description = format!(
        "Stars: `{}★` Length: `{}` (`{}`) Combo: `{}x`\n\
        CS: `{}` HP: `{}` OD: `{}` AR: `{}`",
        round(map.stars),
        sec_to_minsec(map.seconds_total),
        sec_to_minsec(map.seconds_drain),
        map.max_combo.unwrap_or(0),
        round(map.cs),
        round(map.hp),
        round(map.od),
        round(map.ar)
    );
    let image = format!(
        "https://assets.ppy.sh/beatmaps/{}/covers/cover.jpg",
        map.mapset_id
    );
    let footer = Footer::new(format!(
        "You can submit plays until {}",
        (end - ChronoDuration::minutes(5)).format(OSUVS_DATE_FORMAT)
    ));
    let builder = EmbedBuilder::new()
        .title(title)
        .url(url)
        .author(author)
        .description(description)
        .image(image)
        .footer(footer);

    ctx.http
        .create_message(OSUVS_CHANNEL)
        .embeds(&[builder.build()])
        .unwrap()
        .exec()
        .await?;

    Ok(())
}

async fn map_end(ctx: &Context, map_id: u32) -> BotResult<()> {
    info!("Ending osuvs map, sending final message...");
    //? Send message to user in case of error?
    let map = ctx.osu.beatmap().map_id(map_id).await?;
    //? Send message to user in case of error?
    //? Maybe put this in get_osuvs_highscores function itself?
    let highscores: Vec<_> = ctx.database.get_osuvs_highscores(3).await?;
    let mut users: HashMap<_, _> = {
        let user_reqs = highscores
            .iter()
            .map(|(user_id, _)| ctx.osu.user(*user_id).mode(GameMode::STD));

        match try_join_all(user_reqs).await {
            Ok(options) => options.into_iter().map(|u| (u.user_id, u)).collect(),
            Err(why) => {
                unwind_error!(
                    error,
                    why,
                    "Error when retrieving users for osuvs highscores: {}"
                );
                HashMap::new()
            }
        }
    };
    let highscores: Vec<_> = highscores
        .into_iter()
        .map(|(user_id, score)| (users.remove(&user_id).unwrap(), score))
        .collect();

    let title = map_to_string(&map);
    let url = format!("{}b/{}", OSU_BASE, map_id);
    let author = Author::new("Current OsuVS Leaderboard");

    if highscores.len() == 0 {
        let description = "No scores have been submitted this week! I am quite saddened by this :(";
        let builder = EmbedBuilder::new()
            .title(title)
            .url(url)
            .author(author)
            .description(description);
        ctx.http
            .create_message(OSUVS_CHANNEL)
            .embeds(&[builder.build()])
            .unwrap()
            .exec()
            .await?;
        return Ok(());
    }
    let map_path = prepare_beatmap_file(map_id).await?;
    let pp_map = Beatmap::from_path(map_path).await?;
    let mut description = String::new();
    let mut attr_values: HashMap<GameMods, OsuPerformanceAttributes> = HashMap::new();
    let thumbnail = format!("https://a.ppy.sh/{}", highscores[0].0.user_id);

    for ((user, score), i) in highscores.into_iter().zip(0..) {
        let attributes = match attr_values.get(&score.mods) {
            Some(attrs) => attrs.to_owned(),
            None => OsuPP::new(&pp_map).mods(score.mods.bits()).calculate(),
        };

        let max_pp = attributes.pp();
        let pp = OsuPP::new(&pp_map)
            .attributes(attributes.difficulty.clone())
            .mods(score.mods.bits())
            .combo(score.max_combo as usize)
            .misses(score.statistics.count_miss as usize)
            .n300(score.statistics.count_300 as usize)
            .n100(score.statistics.count_100 as usize)
            .n50(score.statistics.count_50 as usize)
            .calculate()
            .pp();
        attr_values.insert(score.mods, attributes);

        let _ = writeln!(
            description,
            "{} {} [{}]({}users/{}): {} [ **{}x**/{}x ] **+{}**\n\
            - **{}**/{}PP - {}% - <t:{}:R>\n",
            EMOTE_MEDALS[i],
            EMOTE_RANKS[&score.grade],
            user.username,
            OSU_BASE,
            user.user_id,
            with_comma_uint(score.score),
            score.max_combo,
            map.max_combo.unwrap_or(0),
            score.mods,
            round(pp as f32),
            round(max_pp as f32),
            score.accuracy,
            &score.created_at.timestamp()
        );
    }
    let image = format!(
        "https://assets.ppy.sh/beatmaps/{}/covers/cover.jpg",
        map.mapset_id
    );
    let builder = EmbedBuilder::new()
        .title(title)
        .url(url)
        .thumbnail(thumbnail)
        .author(author)
        .description(description)
        .image(image);

    ctx.http
        .create_message(OSUVS_CHANNEL)
        .embeds(&[builder.build()])
        .unwrap()
        .exec()
        .await?;

    Ok(())
}
