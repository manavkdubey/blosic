use poise::Context;
use serenity::model::prelude::GuildId;
use songbird::input;
use tokio::io::Result;

pub async fn join_and_play(ctx: Context<'_, (), ()>, guild_id: GuildId, url: &str) -> Result<()> {
    let songbird = songbird::get(&ctx.serenity_context())
        .await
        .expect("Songbird Voice client was not initialized!");

    let source = input::ytdl(&url)
        .await
        .expect("Error sourcing audio from provided URL.");

    let manager = songbird
        .get(guild_id)
        .expect("No Songbird instance found for provided guild ID.");

    manager.play_source(source);

    poise::say_reply(ctx, "Playing...").await?;
    Ok(())
}
