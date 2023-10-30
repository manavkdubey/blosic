use crate::utils::voice_helpers::join_and_play;
use poise::serenity_prelude::id::GuildId;
use tokio::io::Result;

use songbird::{input, Songbird};
#[poise::command(slash_command)]
async fn play(ctx: poise::Context<'_, (), ()>, url: String) -> Result<()> {
    let guild_id = match ctx.guild_id() {
        Some(id) => id,
        None => {
            poise::say_reply(ctx, "This command can only be used in servers.").await?;
            return Ok(());
        }
    };
    join_and_play(ctx, guild_id, &url).await?;
    Ok(())
}
