use crate::{
    utils::{conc_map::SyncRwLockMap, osu::username_to_number},
    BotResult,
};
use cow_utils::CowUtils;
use futures::stream::StreamExt;
use irc::client::prelude::*;

pub struct IrcClient {
    // Tracked users
    pub targets: SyncRwLockMap<u128, bool>,
}

impl IrcClient {
    #[inline]
    pub fn new(targets: SyncRwLockMap<u128, bool>) -> Self {
        IrcClient { targets }
    }

    pub async fn run(
        &self,
        server: &str,
        port: u16,
        nickname: &str,
        password: &str,
    ) -> BotResult<()> {
        let config = Config {
            server: Some(server.to_owned()),
            port: Some(port),
            password: Some(password.to_owned()),
            nickname: Some(nickname.to_owned()),
            channels: vec!["#osu".to_owned()],
            use_tls: Some(false),
            ..Default::default()
        };

        let mut client = Client::from_config(config).await?;
        client.identify()?;

        let mut stream = client.stream()?;

        info!("[IRC] Connected to Bancho");
        debug!("{:?}", self.targets);
        while let Some(msg) = stream.next().await.transpose()? {
            // info!("{:#?}", msg);
            match msg.command {
                Command::JOIN(..) => {
                    if let Some(Prefix::Nickname(mut name, ..)) = msg.prefix {
                        let number: u128 = username_to_number(&name);

                        if matches!(self.targets.read(number).get(), Some(false)) {
                            info!("[IRC] {} now online", name);

                            if let Some(online) = self.targets.write(number).get_mut() {
                                *online = true
                            };
                        }
                    }
                }
                Command::QUIT(..) => {
                    if let Some(Prefix::Nickname(mut name, ..)) = msg.prefix {
                        let number: u128 = username_to_number(&name);

                        if matches!(self.targets.read(number).get(), Some(true)) {
                            info!("[IRC] {} now offline", name);

                            if let Some(online) = self.targets.write(number).get_mut() {
                                *online = false
                            };
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }
}
