use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{
    CommandResult,
    macros::command,
};
use serenity::constants::GATEWAY_VERSION;
use std::time::Instant;
use crate::helpers::global_data::Database;
use crate::helpers::database_helper::DatabaseGuild;

#[command]
#[description = "Pong!"]
#[aliases("pong", "latency")]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    let gateway_url = format!("https://discord.com/api/v{}/gateway", GATEWAY_VERSION);

    // Get latency, Get the gateway URL.
    let now = Instant::now();
    reqwest::get(&gateway_url).await?;
    let get_latency = now.elapsed().as_millis();

    // Post latency, Send a message.
    let now = Instant::now();
    let mut sent_message = msg.channel_id
        .say(&ctx.http, ":hourglass: Calculating latency...").await?;
    let post_latency = now.elapsed().as_millis();

    // println!("{:?}", ctx.data.read().await.get::<Database>().unwrap().list_database_names(None, None).await.unwrap());
    DatabaseGuild::get(ctx, msg.guild_id.unwrap().0).await;

    sent_message.edit(ctx, |m| {
        m.content("");
        m.embed(|e| {
            e.title("Pong! Latency");
            e.description(format!("REST GET: {}ms\nREST POST: {}ms", get_latency, post_latency))
        })
    }).await?;

    Ok(())
}

#[command]
#[description = "Some information about the bot."]
#[aliases("info")]
async fn about(ctx: &Context, msg: &Message) -> CommandResult {
    let avatar_url = ctx.cache.current_user().await.avatar_url();

    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.title("About kBot");
            e.description(format!("**Bot source**\n{}\n**Support server**\n{}",
                                  "https://github.com/kara-b/kBot2", "https://discord.gg/qzGj4En"));
            e.thumbnail(avatar_url.unwrap_or_else(String::new));
            e
        });
        m
    }).await?;

    Ok(())
}