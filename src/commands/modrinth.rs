use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{CommandResult, macros::command, Args};
use urlencoding::encode as url_encode;
use serenity::static_assertions::_core::time::Duration;
use serenity::futures::StreamExt;
use serenity::builder::CreateEmbed;
use serenity::utils::Colour;
use serenity::model::Permissions;
use crate::models::modrinth::*;

#[command]
#[aliases("s")]
pub async fn search(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if args.is_empty() {
        msg.channel_id.say(&ctx.http, ":no_entry_sign: Please provide what to search.").await?;
        return Ok(())
    }

    let limit: i32 = 5;
    let mut api_url = format!("https://api.modrinth.com/api/v1/mod?limit={}&index=relevance&", limit);
    // Append query
    api_url.push_str(&format!("query={}&", url_encode(args.message())));

    // Get the json from the API and handle any errors.
    let json_request = reqwest::get(&api_url).await?;
    let json = match json_request.json::<ModrinthModSearch>().await {
        Ok(json) => json,
        Err(why) => {
            if why.is_decode() {
                msg.channel_id.send_message(&ctx.http, |m| m.embed(|e| {
                    e.title("An error occurred decoding Modrinth's response.")
                        .description(format!("```{}```", why))
                        .color(Colour::RED)
                })).await?;
            } else {
                msg.channel_id.say(&ctx.http, ":no_entry_sign: An error occurred fetching from Modrinth's API.").await?;
            }

            return Ok(())
        }
    };

    // Check if there are enough results
    if json.total_hits < 1 {
        msg.channel_id.say(&ctx.http, ":no_entry_sign: Nothing was found.").await?;
        return Ok(())
    }

    // Send initial message and an integer for the current index
    let mut current_hit: usize = 0;
    let mut message = msg.channel_id.send_message(&ctx.http, |m| m.embed(|embed| {
        let current_mod = &json.hits.get(0).unwrap();
        embed.0 = searched_modrinth_embed(current_mod).0;
        embed
    })).await.unwrap();

    // Emote menu
    message.react(&ctx.http, ReactionType::Unicode(String::from("⬅"))).await?;
    message.react(&ctx.http, ReactionType::Unicode(String::from("➡"))).await?;

    // Check if the bot can manage messages, And if so enable the bot to remove reactions.
    let mut can_manage_messages = false;
    let guild = msg.guild(&ctx).await;
    if guild.is_some() {
        can_manage_messages = msg.guild(&ctx).await.unwrap()
            .user_permissions_in(msg.channel_id, &ctx.http.get_current_user().await?.id)
            .contains(Permissions::MANAGE_MESSAGES);
    }

    let mut reactions_collector = message.await_reactions(&ctx).timeout(Duration::from_secs(5 * 60))
        .author_id(msg.author.id).added(true).removed(!can_manage_messages).await;
    while let Some(reaction) = reactions_collector.next().await {
        // Delete the reaction
        if can_manage_messages {
            reaction.as_inner_ref().delete(&ctx.http).await?;
        }

        let emoji = &reaction.as_inner_ref().emoji.to_string();
        if emoji == "⬅" && current_hit != 0 {
            current_hit -= 1;
        } else if emoji == "➡" && current_hit != (limit - 1) as usize && current_hit != (&json.hits.len() - 1) {
            current_hit += 1;
        }

        // Edit the message with the new index
        &message.edit(&ctx.http, |f| f.embed(|embed| {
            let current_mod = &json.hits.get(current_hit).unwrap();
            embed.0 = searched_modrinth_embed(current_mod).0;
            embed
        })).await;
    }

    // Delete all reactions once done
    if can_manage_messages {
        &message.delete_reactions(&ctx.http).await?;
    }

    Ok(())
}

#[command]
#[aliases("modid", "mod_id", "mod")]
pub async fn id(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if args.is_empty() {
        msg.channel_id.say(&ctx.http, ":no_entry_sign: Please provide what mod id to get.").await?;
        return Ok(())
    }

    let api_url = format!("https://api.modrinth.com/api/v1/mod/{}", url_encode(args.message()));

    // Get the json from the API and handle any errors.
    let json_request = reqwest::get(&api_url).await?;
    let modrinth_mod = match json_request.json::<FullModrinthMod>().await {
        Ok(json) => json,
        Err(why) => {
            if why.is_decode() {
                msg.channel_id.send_message(&ctx.http, |m| m.embed(|e| {
                    e.title("An error occurred decoding Modrinth's response.")
                        .description(format!("```{}```", why))
                        .color(Colour::RED)
                })).await?;
            } else {
                msg.channel_id.say(&ctx.http, ":no_entry_sign: An error occurred fetching from Modrinth's API.").await?;
            }

            return Ok(())
        }
    };

    // Send message
    msg.channel_id.send_message(&ctx.http, |m| m.embed(|embed| {
        embed.0 = full_modrinth_embed(&modrinth_mod).0;
        embed
    })).await.unwrap();

    Ok(())
}

fn searched_modrinth_embed(modrinth_mod: &SearchedModrinthMod) -> CreateEmbed {
    let mut embed = CreateEmbed::default();

    embed.title(&modrinth_mod.title)
        .url(&modrinth_mod.page_url)
        .description(&modrinth_mod.description)
        .author(|f| f.name(&modrinth_mod.author).url(&modrinth_mod.author_url))
        .footer(|f| f.text(format!("id: {}, last modified: {}", &modrinth_mod.mod_id, &modrinth_mod.date_modified)))
        .thumbnail(&modrinth_mod.icon_url)
        .field("Categories", &modrinth_mod.categories.join(", "), true)
        .field("Latest version", &modrinth_mod.latest_version, true)
        .field("Total downloads", &modrinth_mod.downloads, true)
        .color(Colour::from(5083687));

    embed
}

fn full_modrinth_embed(modrinth_mod: &FullModrinthMod) -> CreateEmbed {
    let mut embed = CreateEmbed::default();

    embed.title(&modrinth_mod.title)
        .url(&modrinth_mod.source_url.as_ref().unwrap_or(&"".to_string()))
        .description(&modrinth_mod.description)
        .footer(|f| f.text(format!("id: {}, last modified: {}", &modrinth_mod.id, &modrinth_mod.updated)))
        .thumbnail(&modrinth_mod.icon_url)
        .field("Categories", &modrinth_mod.categories.join(", "), true)
        .field("Latest version", &modrinth_mod.versions.last().unwrap_or(&String::from("None")), true)
        .field("Total downloads", &modrinth_mod.downloads, true)
        .field("Status", &modrinth_mod.status, true)
        .field("Team", &modrinth_mod.team, true)
        .color(Colour::from(5083687));

    embed
}