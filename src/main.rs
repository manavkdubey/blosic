use hyper::{header, Body, Client as Client1, Request};
use hyper_tls::HttpsConnector;
use poise::serenity_prelude::GatewayIntents;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    framework::standard::{
        macros::{command, group},
        Args, CommandResult, StandardFramework,
    },
    model::{channel::Message, gateway::Ready},
};
use songbird::SerenityInit;
use std::env;

#[group]
#[commands(join, leave, play, stop, ask)]
struct General;
#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: MessageContent,
}

#[derive(Deserialize)]
struct MessageContent {
    role: String,
    content: String,
}
struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("Bot is ready with username {}", ready.user.name);
    }
}

#[command]
async fn ask(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let user_input = args.rest();

    let oai_token = "your_open_ai_api_key"; // Use an environment variable or another secure method to store this
    let prompt = user_input.to_string();

    let oai_request = json!({
        "model": "gpt-3.5-turbo",
        "messages": [{"role": "user", "content": prompt}],
        "max_tokens": 100
    });

    let client = Client1::builder().build::<_, hyper::Body>(HttpsConnector::new());

    let req = Request::post("https://api.openai.com/v1/chat/completions")
        .header(header::CONTENT_TYPE, "application/json")
        .header("Authorization", format!("Bearer {}", oai_token))
        .body(Body::from(oai_request.to_string()))
        .expect("Failed to build request");

    let res = client.request(req).await.expect("Request failed");
    let body = hyper::body::to_bytes(res.into_body())
        .await
        .expect("Failed to read response body");

    let response: OpenAIResponse = serde_json::from_slice(&body).expect("Failed to parse response");

    if let Some(choice) = response.choices.get(0) {
        msg.channel_id
            .say(&ctx.http, &choice.message.content)
            .await?;
    } else {
        msg.channel_id
            .say(&ctx.http, "No response received.")
            .await?;
    }

    Ok(())
}

#[command]
async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialization.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        handler.stop();
    } else {
        msg.channel_id
            .say(&ctx.http, "Not in a voice channel or no song playing.")
            .await?;
    }

    Ok(())
}

#[command]
async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    // Get the voice channel of the user
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            msg.reply(ctx, "You are not in a voice channel.").await?;
            return Ok(());
        }
    };

    // Get the songbird manager
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialization.")
        .clone();

    // Join the user's voice channel
    let (handler_lock, _) = manager.join(guild_id, connect_to).await;
    let mut handler = handler_lock.lock().await;

    // Get the URL to play (you'd ideally have some error handling here in a real application)
    let url = args.single::<String>()?;

    // Play the given URL
    if let Ok(source) = songbird::ytdl(&url).await {
        handler.play_source(source.into());
        msg.channel_id
            .say(&ctx.http, "Playing the requested song!")
            .await?;
    } else {
        msg.channel_id
            .say(&ctx.http, "Error sourcing the media.")
            .await?;
    }

    Ok(())
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "Pong!").await?;

    Ok(())
}
#[command]
async fn join(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            msg.reply(ctx, "Not in a voice channel.").await?;
            return Ok(());
        }
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialization.")
        .clone();

    manager.join(guild_id, connect_to).await;

    Ok(())
}

#[command]
async fn leave(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialization.")
        .clone();

    manager.remove(guild_id);

    Ok(())
}

#[tokio::main]
async fn main() {
    let token = "YOUR_DISCORD_BOT_TOKEN";

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~"))
        .group(&GENERAL_GROUP);

    let mut client = Client::builder(token, GatewayIntents::all())
        .event_handler(Handler)
        .framework(framework)
        .register_songbird()
        .await
        .expect("Error creating client");

    let _ = client
        .start()
        .await
        .map_err(|why| println!("Client ended: {:?}", why));
}
