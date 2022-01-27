// use crate::commands::osuvs::{map_end, map_start};

// use chrono::{DateTime, Duration as ChronoDuration, Utc};
// use futures::{
//     future::FutureExt,
//     stream::{FuturesUnordered, StreamExt},
// };
// use itertools::Itertools;
// use rosu_v2::prelude::{GameMode, GameMods, Grade, Score};
// use std::{collections::HashMap, sync::Arc};
// use tokio::time::{interval, Duration};

// const OSUVS_TRACK_INTERVAL: u64 = 300;

// pub async fn osu_tracking(http: Arc<Http>, data: Arc<RwLock<TypeMap>>) {
//     // Users might be contained multiple times but w/e
//     let irc = {
//         let data = data.read().await;

//         Arc::clone(data.get::<Irc>().unwrap())
//     };

//     let mut interval = interval(Duration::from_secs(OSUVS_TRACK_INTERVAL));
//     interval.tick().await;

//     let mut user_ids = HashMap::with_capacity(irc.targets.len());

//     loop {
//         interval.tick().await;

//         let (map_id, start, end) = match curr_osuvs_id(&data).await {
//             Some(tuple) => tuple,
//             None => continue,
//         };

//         let now = Utc::now();

//         if now - start < ChronoDuration::seconds(OSUVS_TRACK_INTERVAL as i64) {
//             if let Err(why) = map_start(&http, &data, map_id, end).await {
//                 unwind_error!(error, "Error while announcing new osuvs map: {}", why);
//             }
//         }

//         let mut users = Vec::new();

//         {
//             let data = data.read().await;
//             let osu = data.get::<Osu>().unwrap();

//             for name in irc.online.iter() {
//                 if !user_ids.contains_key(name.as_str()) {
//                     match osu.user(name.as_str()).await {
//                         Ok(user) => {
//                             user_ids.insert(name.to_owned(), user.user_id);
//                         }
//                         Err(why) => {
//                             let why = anyhow::Error::new(why);
//                             unwind_error!(warn, "Failed to add user id`: {}", why);

//                             continue;
//                         }
//                     }
//                 }

//                 users.push(user_ids[name.as_str()]);
//             }
//         }

//         debug!("[Track] {} users: {:?}", users.len(), users);
//         loop_iteration(map_id, &data, &users, start).await;

//         if end - now < ChronoDuration::seconds(OSUVS_TRACK_INTERVAL as i64) {
//             if let Err(why) = map_end(&http, data.clone(), map_id).await {
//                 unwind_error!(error, "Error while announcing end of osuvs map: {}", why);
//             }
//         }
//     }
// }

// async fn curr_osuvs_id(data: &RwLock<TypeMap>) -> Option<(u32, DateTime<Utc>, DateTime<Utc>)> {
//     let data = data.read().await;
//     let mysql = data.get::<MySQL>().unwrap();

//     match mysql.get_curr_osuvs_id() {
//         // Got current map_id
//         Ok(Some(tuple)) => Some(tuple),
//         // OsuVS currently not running, skip loop iteration
//         Ok(None) => None,
//         Err(why) => {
//             unwind_error!(error, "Error while getting current osuvs map id: {}", why);

//             None
//         }
//     }
// }

// async fn loop_iteration(map_id: u32, data: &RwLock<TypeMap>, users: &[u32], start: DateTime<Utc>) {
//     // Request the last 50 recents scores for all online tracked users
//     let scores: HashMap<u32, Vec<Score>> = {
//         let data = data.read().await;
//         let osu = data.get::<Osu>().unwrap();

//         users
//             .iter()
//             .map(|&user_id| {
//                 osu.user_scores(user_id)
//                     .recent()
//                     .mode(GameMode::STD)
//                     .limit(50)
//                     .map(move |res| (user_id, res))
//             })
//             .collect::<FuturesUnordered<_>>()
//             .filter_map(|(user_id, res)| async move {
//                 match res {
//                     Ok(scores) => Some((user_id, scores)),
//                     Err(why) => {
//                         let why = anyhow::Error::new(why);
//                         unwind_error!(warn, "Error while requesting tracked user: {}", why);

//                         None
//                     }
//                 }
//             })
//             .collect()
//             .await
//     };

//     // Map each user to a vec containing the best score
//     // on the osuvs map for each played mod
//     let recent_best: HashMap<_, _> = scores
//         .into_iter()
//         .filter_map(|(_, scores)| {
//             let user_id = scores.first().map(|s| s.user_id)?;

//             let scores: Vec<_> = scores
//                 .into_iter()
//                 .filter(|s| s.map.as_ref().unwrap().map_id == map_id)
//                 .filter(|s| s.grade(None) != Grade::F)
//                 .filter(|s| s.created_at >= start)
//                 .filter(|s| !s.mods.contains(GameMods::ScoreV2))
//                 .map(|s| (s.mods.bits(), s))
//                 .sorted_by(|(m1, _), (m2, _)| m1.cmp(m2))
//                 .group_by(|(mods, _)| *mods)
//                 .into_iter()
//                 .flat_map(|(_, group)| {
//                     group
//                         .sorted_by(|(_, s1), (_, s2)| s2.score.cmp(&s1.score))
//                         .next()
//                 })
//                 .collect();

//             (!scores.is_empty()).then(|| (user_id, scores))
//         })
//         .collect();

//     // If no one played the map, skip loop iteration
//     if recent_best.is_empty() {
//         return;
//     }

//     let total_best = {
//         let data = data.read().await;
//         let mysql = data.get::<MySQL>().unwrap();

//         match mysql.get_osuvs_highscores(map_id) {
//             Ok(highscores) => highscores,
//             Err(why) => {
//                 unwind_error!(error, "Error while getting OsuVS highscores: {}", why);

//                 return;
//             }
//         }
//     };

//     for (user, scores) in recent_best {
//         // Check if the new (mods,score) tuples are better than the previous ones
//         let new_scores: Vec<_> = if let Some(best) = total_best.get(&user) {
//             scores
//                 .into_iter()
//                 .filter_map(|(mods, score)| match best.get(&mods) {
//                     Some(best_score) => (score.score > best_score.score).then(|| (mods, score)),
//                     None => Some((mods, score)),
//                 })
//                 .collect()
//         } else {
//             scores
//         };

//         if !new_scores.is_empty() {
//             let data = data.read().await;
//             let mysql = data.get::<MySQL>().unwrap();

//             if let Err(why) = mysql.insert_osuvs_highscores(map_id, user, new_scores) {
//                 unwind_error!(error, "Error while inserting new OsuVS scores: {}", why);
//             }
//         }
//     }
// }
