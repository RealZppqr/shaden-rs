use anyhow::Result;
use serenity::prelude::*;
use serenity::all::{ApplicationCommandInteraction, InteractionResponseType};
use crate::services::Database;

pub async fn handle_join_rewards(ctx: &Context, command: &ApplicationCommandInteraction, database: &Database) -> Result<()> {
    let discord_id = command.user.id.0;
    let action = command.data.options.get(0)
        .and_then(|opt| opt.value.as_ref())
        .and_then(|val| val.as_str())
        .unwrap_or("claim");

    match action {
        "claim" => claim_join_reward(ctx, command, database, discord_id).await,
        _ => {
            command.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message.content("Invalid action").ephemeral(true)
                    })
            }).await?;
            Ok(())
        }
    }
}

async fn claim_join_reward(ctx: &Context, command: &ApplicationCommandInteraction, database: &Database, discord_id: u64) -> Result<()> {
    let server_invite = command.data.options.iter()
        .find(|opt| opt.name == "server_invite")
        .and_then(|opt| opt.value.as_ref())
        .and_then(|val| val.as_str());

    if server_invite.is_none() {
        command.create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.content("Please provide a server invite").ephemeral(true)
                })
        }).await?;
        return Ok(());
    }

    // Placeholder implementation - in real bot, verify Discord server membership
    let user = database.get_user(discord_id).await?;
    if user.is_none() {
        command.create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.content("You need to login first! Use `/login`").ephemeral(true)
                })
        }).await?;
        return Ok(());
    }

    let mut user = user.unwrap();
    user.add_coins(50); // Award 50 coins for joining
    database.update_user(&user).await?;

    command.create_interaction_response(&ctx.http, |response| {
        response
            .kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| {
                message
                    .embed(|embed| {
                        embed
                            .title("🎉 Join Reward Claimed!")
                            .description("You earned **50 coins** for joining a partner server!\n\nThank you for supporting our community!")
                            .color(0x00ff00)
                    })
                    .ephemeral(true)
            })
    }).await?;

    Ok(())
}
