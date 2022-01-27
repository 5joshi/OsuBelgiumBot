use twilight_model::user::User;

pub fn user_avatar(user: &User) -> String {
    match user.avatar {
        Some(ref hash) if hash.starts_with("a_") => {
            format!("https://cdn.discordapp.com/avatars/{}/{hash}.gif", user.id)
        }
        Some(ref hash) => format!("https://cdn.discordapp.com/avatars/{}/{hash}.png", user.id),
        None => format!(
            "https://cdn.discordapp.com/embed/avatars/{}.png",
            user.discriminator
        ),
    }
}
