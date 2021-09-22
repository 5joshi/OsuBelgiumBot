use std::{borrow::Cow, mem};

use crate::{context::Context, utils::MessageBuilder, BotResult, Error};

use twilight_model::{
    application::{
        callback::{CallbackData, InteractionResponse},
        interaction::{application_command::CommandDataOption, ApplicationCommand},
    },
    id::UserId,
};

#[async_trait]
pub trait ApplicationCommandExt {
    fn user_id(&self) -> BotResult<UserId>;
    fn username(&self) -> BotResult<&str>;
    fn yoink_options(&mut self) -> Vec<CommandDataOption>;
    async fn create_message(&self, ctx: &Context, builder: MessageBuilder<'_>) -> BotResult<()>;
}

#[async_trait]
impl ApplicationCommandExt for ApplicationCommand {
    fn user_id(&self) -> BotResult<UserId> {
        self.member
            .as_ref()
            .and_then(|member| member.user.as_ref())
            .or_else(|| self.user.as_ref())
            .map(|user| user.id)
            .ok_or(Error::MissingSlashAuthor)
    }

    fn username(&self) -> BotResult<&str> {
        self.member
            .as_ref()
            .and_then(|member| member.user.as_ref())
            .or_else(|| self.user.as_ref())
            .map(|user| user.name.as_str())
            .ok_or(Error::MissingSlashAuthor)
    }

    fn yoink_options(&mut self) -> Vec<CommandDataOption> {
        mem::take(&mut self.data.options)
    }

    //TODO: ephemeral flags in builder
    async fn create_message(&self, ctx: &Context, builder: MessageBuilder<'_>) -> BotResult<()> {
        let response = InteractionResponse::ChannelMessageWithSource(CallbackData {
            allowed_mentions: None,
            components: None,
            content: builder.content.map(Cow::into_owned),
            embeds: builder.embed.map_or_else(Vec::new, |e| vec![e]),
            flags: None,
            tts: None,
        });

        ctx.http
            .interaction_callback(self.id, &self.token, &response)
            .exec()
            .await?;

        Ok(())
    }
}
