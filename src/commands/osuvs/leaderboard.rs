use std::fmt::Write;
use std::sync::Arc;

use futures::future::try_join_all;
use hashbrown::HashMap;
use rosu_pp::{osu::OsuPerformanceAttributes, Beatmap, OsuPP};
use rosu_v2::prelude::{GameMode, GameMods};
use twilight_model::application::interaction::ApplicationCommand;

use crate::{
    context::Context,
    error::BotResult,
    utils::{
        numbers::{round, with_comma_uint},
        osu::{map_to_string, prepare_beatmap_file},
        ApplicationCommandExt, Author, EmbedBuilder, MessageBuilder, EMOTE_RANKS, OSU_BASE,
    },
};

pub async fn leaderboard(ctx: Arc<Context>, command: ApplicationCommand) -> BotResult<()> {
    info!("Displaying current osuvs info...");
    match ctx.database.get_curr_osuvs_map().await {
        Some((map_id, _, _)) => {
            //? Send message to user in case of error?
            let map = ctx.osu.beatmap().map_id(map_id).await?;
            //? Send message to user in case of error?
            //? Maybe put this in get_osuvs_highscores function itself?
            let highscores: Vec<_> = ctx.database.get_osuvs_highscores(10).await?;
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
                let description = "No scores have been submitted yet! This is very sad :(";
                let builder = EmbedBuilder::new()
                    .title(title)
                    .url(url)
                    .author(author)
                    .description(description);
                return command.create_message(&ctx, builder).await;
            }
            let map_path = prepare_beatmap_file(map_id).await?;
            let pp_map = Beatmap::from_path(map_path).await?;
            let mut description = String::new();
            let mut attr_values: HashMap<GameMods, OsuPerformanceAttributes> = HashMap::new();
            let thumbnail = format!("https://a.ppy.sh/{}", highscores[0].0.user_id);

            for ((user, score), i) in highscores.into_iter().zip(1..) {
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
                    "**{}.** {} [{}]({}users/{}): {} [ **{}x**/{}x ] **+{}**\n\
                - **{}**/{}PP - {}% - <t:{}:R>",
                    i,
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
                    round(score.accuracy),
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

            command.create_message(&ctx, builder).await
        }
        None => {
            let builder = MessageBuilder::new().error("There is currently no ongoing OsuVS!");

            command.create_message(&ctx, builder).await
        }
    }
}
