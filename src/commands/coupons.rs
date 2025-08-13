use anyhow::Result;
use serenity::prelude::*;
use serenity::all::{ApplicationCommandInteraction, InteractionResponseType};
use crate::services::Database;
use crate::models::Resources;

pub async fn handle_coupons(ctx: &Context, command: &ApplicationCommandInteraction, database: &Database) -> Result<()> {
    let discord_id = command.user.id.0;
    let action = command.data.options.get(0)
        .and_then(|opt| opt.value.as_ref())
        .and_then(|val| val.as_str())
        .unwrap_or("redeem");

    match action {
        "redeem" => redeem_coupon(ctx, command, database, discord_id).await,
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

async fn redeem_coupon(ctx: &Context, command: &ApplicationCommandInteraction, database: &Database, discord_id: u64) -> Result<()> {
    let code = command.data.options.iter()
        .find(|opt| opt.name == "code")
        .and_then(|opt| opt.value.as_ref())
        .and_then(|val| val.as_str());

    if code.is_none() {
        command.create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.content("Please provide a coupon code").ephemeral(true)
                })
        }).await?;
        return Ok(());
    }

    let code = code.unwrap();

    // Get user and coupon
    let user = database.get_user(discord_id).await?;
    let coupon = database.get_coupon(code).await?;

    match (user, coupon) {
        (Some(mut user), Some(mut coupon)) => {
            if !coupon.can_be_used(discord_id) {
                command.create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message.content("This coupon is invalid, expired, or you've already used it").ephemeral(true)
                        })
                }).await?;
                return Ok(());
            }

            // Use the coupon
            coupon.use_coupon(discord_id);
            user.add_coins(coupon.coins);

            // Add resources if any
            if let Some(resources) = &coupon.resources {
                user.resources.ram += resources.ram;
                user.resources.cpu += resources.cpu;
                user.resources.disk += resources.disk;
                user.resources.databases += resources.databases;
                user.resources.allocations += resources.allocations;
                user.resources.backups += resources.backups;
            }

            // Update database
            database.update_user(&user).await?;
            database.update_coupon(&coupon).await?;

            let mut description = format!("Successfully redeemed coupon **{}**!\n\n", code);
            description.push_str(&format!("💰 Coins: +{}\n", coupon.coins));
            
            if let Some(resources) = &coupon.resources {
                if resources.ram > 0 { description.push_str(&format!("🖥️ RAM: +{}MB\n", resources.ram)); }
                if resources.cpu > 0 { description.push_str(&format!("⚡ CPU: +{}%\n", resources.cpu)); }
                if resources.disk > 0 { description.push_str(&format!("💾 Disk: +{}MB\n", resources.disk)); }
                if resources.databases > 0 { description.push_str(&format!("🗄️ Databases: +{}\n", resources.databases)); }
                if resources.allocations > 0 { description.push_str(&format!("🌐 Allocations: +{}\n", resources.allocations)); }
                if resources.backups > 0 { description.push_str(&format!("💾 Backups: +{}\n", resources.backups)); }
            }

            command.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message
                            .embed(|embed| {
                                embed
                                    .title("🎉 Coupon Redeemed!")
                                    .description(description)
                                    .color(0x00ff00)
                            })
                            .ephemeral(true)
                    })
            }).await?;
        }
        (None, _) => {
            command.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message.content("You need to login first! Use `/login`").ephemeral(true)
                    })
            }).await?;
        }
        (_, None) => {
            command.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message.content("Invalid coupon code").ephemeral(true)
                    })
            }).await?;
        }
    }

    Ok(())
}
