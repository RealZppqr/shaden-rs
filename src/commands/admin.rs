use anyhow::Result;
use serenity::prelude::*;
use serenity::all::{ApplicationCommandInteraction, InteractionResponseType};
use crate::services::Database;
use crate::config::Config;
use crate::models::{Coupon, Resources};

pub async fn handle_admin(ctx: &Context, command: &ApplicationCommandInteraction, database: &Database, config: &Config) -> Result<()> {
    let discord_id = command.user.id.0;
    
    // Check if user is admin
    if !config.is_admin(discord_id) {
        command.create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.content("❌ You don't have permission to use admin commands").ephemeral(true)
                })
        }).await?;
        return Ok(());
    }

    let subcommand = command.data.options.get(0)
        .and_then(|opt| opt.name.as_str());

    match subcommand {
        Some("coins") => handle_admin_coins(ctx, command, database).await,
        Some("resources") => handle_admin_resources(ctx, command, database).await,
        Some("coupons") => handle_admin_coupons(ctx, command, database, discord_id).await,
        Some("stats") => show_admin_stats(ctx, command, database).await,
        _ => {
            command.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message.content("Invalid admin command").ephemeral(true)
                    })
            }).await?;
            Ok(())
        }
    }
}

async fn handle_admin_coins(ctx: &Context, command: &ApplicationCommandInteraction, database: &Database) -> Result<()> {
    let action = command.data.options.get(0)
        .and_then(|opt| opt.options.get(0))
        .and_then(|opt| opt.value.as_ref())
        .and_then(|val| val.as_str());

    let target_user = command.data.options.get(0)
        .and_then(|opt| opt.options.iter().find(|o| o.name == "user"))
        .and_then(|opt| opt.value.as_ref())
        .and_then(|val| val.as_str())
        .and_then(|s| s.parse::<u64>().ok());

    let amount = command.data.options.get(0)
        .and_then(|opt| opt.options.iter().find(|o| o.name == "amount"))
        .and_then(|opt| opt.value.as_ref())
        .and_then(|val| val.as_i64());

    if target_user.is_none() || amount.is_none() {
        command.create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.content("Please provide both user and amount").ephemeral(true)
                })
        }).await?;
        return Ok(());
    }

    let target_user = target_user.unwrap();
    let amount = amount.unwrap();

    let user = database.get_or_create_user(target_user).await?;
    let mut user = user;

    match action {
        Some("set") => {
            user.coins = amount;
            database.update_user(&user).await?;
            
            command.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message
                            .embed(|embed| {
                                embed
                                    .title("✅ Admin Action Complete")
                                    .description(format!("Set <@{}>'s coins to **{}**", target_user, amount))
                                    .color(0x00ff00)
                            })
                            .ephemeral(true)
                    })
            }).await?;
        }
        Some("add") => {
            user.add_coins(amount);
            database.update_user(&user).await?;
            
            command.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message
                            .embed(|embed| {
                                embed
                                    .title("✅ Admin Action Complete")
                                    .description(format!("Added **{}** coins to <@{}>\nNew balance: **{}**", amount, target_user, user.coins))
                                    .color(0x00ff00)
                            })
                            .ephemeral(true)
                    })
            }).await?;
        }
        Some("remove") => {
            if let Err(e) = user.deduct_coins(amount) {
                command.create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message.content(format!("Error: {}", e)).ephemeral(true)
                        })
                }).await?;
                return Ok(());
            }
            
            database.update_user(&user).await?;
            
            command.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message
                            .embed(|embed| {
                                embed
                                    .title("✅ Admin Action Complete")
                                    .description(format!("Removed **{}** coins from <@{}>\nNew balance: **{}**", amount, target_user, user.coins))
                                    .color(0x00ff00)
                            })
                            .ephemeral(true)
                    })
            }).await?;
        }
        _ => {
            command.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message.content("Invalid coins action").ephemeral(true)
                    })
            }).await?;
        }
    }

    Ok(())
}

async fn handle_admin_resources(ctx: &Context, command: &ApplicationCommandInteraction, database: &Database) -> Result<()> {
    // Similar implementation to handle_admin_coins but for resources
    command.create_interaction_response(&ctx.http, |response| {
        response
            .kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| {
                message.content("Admin resources management not yet implemented").ephemeral(true)
            })
    }).await?;

    Ok(())
}

async fn handle_admin_coupons(ctx: &Context, command: &ApplicationCommandInteraction, database: &Database, admin_id: u64) -> Result<()> {
    let action = command.data.options.get(0)
        .and_then(|opt| opt.options.get(0))
        .and_then(|opt| opt.value.as_ref())
        .and_then(|val| val.as_str());

    match action {
        Some("create") => create_coupon(ctx, command, database, admin_id).await,
        Some("revoke") => revoke_coupon(ctx, command, database).await,
        _ => {
            command.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message.content("Invalid coupon action").ephemeral(true)
                    })
            }).await?;
            Ok(())
        }
    }
}

async fn create_coupon(ctx: &Context, command: &ApplicationCommandInteraction, database: &Database, admin_id: u64) -> Result<()> {
    let code = command.data.options.get(0)
        .and_then(|opt| opt.options.iter().find(|o| o.name == "code"))
        .and_then(|opt| opt.value.as_ref())
        .and_then(|val| val.as_str());

    let coins = command.data.options.get(0)
        .and_then(|opt| opt.options.iter().find(|o| o.name == "coins"))
        .and_then(|opt| opt.value.as_ref())
        .and_then(|val| val.as_i64());

    if code.is_none() || coins.is_none() {
        command.create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.content("Please provide both code and coins amount").ephemeral(true)
                })
        }).await?;
        return Ok(());
    }

    let code = code.unwrap();
    let coins = coins.unwrap();

    // Check if coupon already exists
    if database.get_coupon(code).await?.is_some() {
        command.create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.content("A coupon with this code already exists").ephemeral(true)
                })
        }).await?;
        return Ok(());
    }

    let coupon = Coupon::new(
        code.to_string(),
        coins,
        None, // No resources for now
        None, // No usage limit
        None, // No expiry
        admin_id,
    );

    database.create_coupon(&coupon).await?;

    command.create_interaction_response(&ctx.http, |response| {
        response
            .kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| {
                message
                    .embed(|embed| {
                        embed
                            .title("✅ Coupon Created")
                            .description(format!("Created coupon **{}** worth **{} coins**", code, coins))
                            .color(0x00ff00)
                    })
                    .ephemeral(true)
            })
    }).await?;

    Ok(())
}

async fn revoke_coupon(ctx: &Context, command: &ApplicationCommandInteraction, database: &Database) -> Result<()> {
    let code = command.data.options.get(0)
        .and_then(|opt| opt.options.iter().find(|o| o.name == "code"))
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

    if database.get_coupon(code).await?.is_none() {
        command.create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.content("Coupon not found").ephemeral(true)
                })
        }).await?;
        return Ok(());
    }

    database.delete_coupon(code).await?;

    command.create_interaction_response(&ctx.http, |response| {
        response
            .kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| {
                message
                    .embed(|embed| {
                        embed
                            .title("✅ Coupon Revoked")
                            .description(format!("Revoked coupon **{}**", code))
                            .color(0xff0000)
                    })
                    .ephemeral(true)
            })
    }).await?;

    Ok(())
}

async fn show_admin_stats(ctx: &Context, command: &ApplicationCommandInteraction, database: &Database) -> Result<()> {
    // Implementation for showing system statistics
    command.create_interaction_response(&ctx.http, |response| {
        response
            .kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| {
                message
                    .embed(|embed| {
                        embed
                            .title("📊 System Statistics")
                            .description("System stats functionality not yet implemented")
                            .color(0x00ff00)
                    })
                    .ephemeral(true)
            })
    }).await?;

    Ok(())
}
