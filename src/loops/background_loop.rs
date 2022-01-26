use crate::{
    context::Context,
    utils::{EmbedBuilder, APPROVE_CHANNEL, SERVER_ID, TOP_ROLE_ID},
};

use chrono::{Duration, Utc};
use rosu_v2::model::GameMode;
use std::sync::Arc;
use tokio::time::{interval, Duration as TokioDuration};

const OSU_TOP_COUNT: usize = 10;
const MNA_TOP_COUNT: usize = 5;
const TKO_TOP_COUNT: usize = 5;
const CTB_TOP_COUNT: usize = 5;

pub async fn background_loop(ctx: Arc<Context>) {
    // Once per day
    let mut interval = interval(TokioDuration::from_secs(60 * 60 * 24));
    interval.tick().await;

    loop {
        interval.tick().await;
        top_role(&ctx).await;
        not_checked_role(&ctx).await;
        info!("Handled unchecked members and top role distribution");
    }
}

async fn top_role(ctx: &Context) {
    // Handle Top role
    let (std, mna, tko, ctb) = {
        let osu = &ctx.osu;

        let std_fut = osu.performance_rankings(GameMode::STD).country("be");
        let mna_fut = osu.performance_rankings(GameMode::MNA).country("be");
        let tko_fut = osu.performance_rankings(GameMode::TKO).country("be");
        let ctb_fut = osu.performance_rankings(GameMode::CTB).country("be");

        tokio::join!(std_fut, mna_fut, tko_fut, ctb_fut)
    };

    let mut all = Vec::with_capacity(OSU_TOP_COUNT + MNA_TOP_COUNT + TKO_TOP_COUNT + CTB_TOP_COUNT);

    for (mode, name, count) in [
        (std, "std", OSU_TOP_COUNT),
        (mna, "mna", MNA_TOP_COUNT),
        (tko, "tko", TKO_TOP_COUNT),
        (ctb, "ctb", CTB_TOP_COUNT),
    ] {
        match mode {
            Ok(rankings) => {
                let iter = rankings.ranking.into_iter().take(count).map(|u| u.user_id);

                all.extend(iter);
            }
            Err(why) => {
                unwind_error!(warn, why, "Could not get top 50 for {}: {}", name);
            }
        }
    }

    if all.is_empty() {
        info!("Skipping Top role management due to API issues");

        return;
    }

    match ctx.database.get_manual_links().await {
        Ok(links) => {
            let guild_id = SERVER_ID;
            let role = TOP_ROLE_ID;

            let req = ctx.http.guild_members(guild_id).limit(1000).unwrap().exec();
            let members = match req.await {
                Ok(res) => res.models().await.unwrap_or_else(|why| {
                    unwind_error!(
                        warn,
                        why,
                        "Could not deserialize guild members for top role: {}"
                    );
                    Vec::new()
                }),
                Err(why) => {
                    unwind_error!(warn, why, "Could not get guild members for top role: {}");
                    Vec::new()
                }
            };

            // Check all guild's members
            for member in members {
                let id = links.get(&member.user.id);

                // If name is contained in manual links
                if let Some(osu_id) = id.copied() {
                    // If member already has top role, check if it remains
                    if member.roles.contains(&role) {
                        if !all.contains(&osu_id) {
                            let req = ctx
                                .http
                                .remove_guild_member_role(guild_id, member.user.id, TOP_ROLE_ID)
                                .exec()
                                .await;

                            if let Err(why) = req {
                                unwind_error!(
                                    error,
                                    why,
                                    "Could not remove top role from member: {}"
                                );
                            } else {
                                info!("Removed 'Top' role from member {}", member.user.name);
                            }
                        }
                    // Member does not have top role yet, 'all' contains the user ID
                    } else if all.contains(&osu_id) {
                        let req = ctx
                            .http
                            .add_guild_member_role(guild_id, member.user.id, TOP_ROLE_ID)
                            .exec()
                            .await;

                        if let Err(why) = req {
                            unwind_error!(error, why, "Could not add top role to member: {}");
                        } else {
                            info!("Added 'Top' role to member {}", member.user.name);
                        }
                    }
                }
            }
        }
        Err(why) => unwind_error!(warn, why, "Could not get manual links from DB: {}"),
    }
}

async fn not_checked_role(ctx: &Context) {
    let day_limit = 10;
    let database = &ctx.database;

    // Handle Not Checked role
    match database.get_unchecked_members().await {
        Ok(members) => {
            let limit_date = Utc::now() - Duration::days(day_limit);
            let guild_id = SERVER_ID;

            for (user_id, join_date) in members {
                if limit_date > join_date {
                    let req = ctx.http.remove_guild_member(guild_id, user_id).exec().await;

                    if let Err(why) = req {
                        warn!(
                            "Could not kick member {} who joined {}: {}",
                            user_id, join_date, why
                        );
                    } else {
                        let content = format!(
                            "Kicking member <@!{user_id}> for being unchecked after {day_limit} days",
                        );
                        let embed = EmbedBuilder::new().description(content).build();
                        ctx.http
                            .create_message(APPROVE_CHANNEL)
                            .embeds(&[embed])
                            .unwrap()
                            .exec()
                            .await;
                    }
                }
            }
        }
        Err(why) => unwind_error!(warn, why, "Could not get unchecked members from DB: {}"),
    }
}
