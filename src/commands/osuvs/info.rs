use std::sync::Arc;

use chrono::Duration;
use twilight_model::application::interaction::ApplicationCommand;

use crate::{
    context::Context,
    error::BotResult,
    utils::{
        datetime::sec_to_minsec, numbers::round, osu::map_to_string, ApplicationCommandExt, Author,
        EmbedBuilder, Footer, MessageBuilder, OSUVS_DATE_FORMAT, OSU_BASE,
    },
};

pub async fn info(ctx: Arc<Context>, command: ApplicationCommand) -> BotResult<()> {
    info!("Displaying current osuvs info...");
    match ctx.database.get_curr_osuvs_map().await {
        Some((map_id, start_date, end_date)) => {
            //? Send message to user in case of error?
            let map = ctx.osu.beatmap().map_id(map_id).await?;
            let title = map_to_string(&map);
            let url = format!("{}b/{}", OSU_BASE, map_id);
            let author = Author::new(format!(
                "This OsuVS started on {}",
                start_date.format(OSUVS_DATE_FORMAT)
            ));
            let description = format!(
                "Stars: `{}★` Length: `{}` (`{}`) Combo: `{}x`\n\
                CS: `{}` HP: `{}` OD: `{}` AR: `{}`",
                round(map.stars),
                sec_to_minsec(map.seconds_total),
                sec_to_minsec(map.seconds_drain),
                map.max_combo.unwrap(),
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
                (end_date - Duration::minutes(5)).format(OSUVS_DATE_FORMAT)
            ));
            let builder = EmbedBuilder::new()
                .title(title)
                .url(url)
                .author(author)
                .description(description)
                .image(image)
                .footer(footer);

            command.create_message(&ctx, builder).await
        }
        None => {
            let builder = MessageBuilder::new().error("There is currently no ongoing OsuVS!");

            command.create_message(&ctx, builder).await
        }
    }
}
