use anyhow::Result;
use serenity::prelude::*;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::application::interaction::InteractionResponseType;
use crate::services::Database;
use crate::errors::BotError;
use crate::config::Config;

pub async fn login(ctx: &Context, command: &ApplicationCommandInteraction, database: &Database) -> Result<()> {
    let discord_id = command.user.id.0;
    
    match database.get_or_create_user(discord_id).await {
        Ok(user) => {
            command.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message
                            .embed(|embed| {
                                embed
                                    .title("Welcome to Shaden-RS!")
                                    .description(format!("Successfully logged in!\n\n**Your Stats:**\n💰 Coins: {}\n🖥️ RAM: {}MB\n⚡ CPU: {}%\n💾 Disk: {}MB", 
                                        user.coins, user.resources.ram, user.resources.cpu, user.resources.disk))
                                    .color(0x00ff00)
                            })
                            .ephemeral(true)
                    })
            }).await?;
        }
        Err(e) => {
            command.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message
                            .content(format!("Failed to login: {}", e))
                            .ephemeral(true)
                    })
            }).await?;
        }
    }

    Ok(())
}

pub async fn handle_coins(ctx: &Context, command: &ApplicationCommandInteraction, database: &Database) -> Result<()> {
    let discord_id = command.user.id.0;
    let action = command.data.options.get(0)
        .and_then(|opt| opt.value.as_ref())
        .and_then(|val| val.as_str())
        .unwrap_or("balance");

    match action {
        "balance" => show_balance(ctx, command, database, discord_id).await,
        "earn-afk" => earn_afk(ctx, command, database, discord_id).await,
        "earn-linkvertise" => earn_linkvertise(ctx, command, database, discord_id).await,
        "gift" => gift_coins(ctx, command, database, discord_id).await,
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

pub async fn transfer_coins(ctx: &Context, command: &ApplicationCommandInteraction, database: &Database, config: &Config) -> Result<()> {
    if !config.enable_transfer {
        command.create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.content("❌ Coin transfers are currently disabled by administrators.").ephemeral(true)
                })
        }).await?;
        return Ok(());
    }

    let discord_id = command.user.id.0;
    
    let target_user = command.data.options.iter()
        .find(|opt| opt.name == "user")
        .and_then(|opt| opt.value.as_ref())
        .and_then(|val| val.as_str())
        .and_then(|s| s.parse::<u64>().ok());

    let amount = command.data.options.iter()
        .find(|opt| opt.name == "amount")
        .and_then(|opt| opt.value.as_ref())
        .and_then(|val| val.as_i64())
        .unwrap_or(0);

    if target_user.is_none() || amount <= 0 {
        command.create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.content("❌ Please specify a valid user and amount (minimum 1 coin).").ephemeral(true)
                })
        }).await?;
        return Ok(());
    }

    let target_user = target_user.unwrap();
    if target_user == discord_id {
        command.create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.content("❌ You cannot transfer coins to yourself!").ephemeral(true)
                })
        }).await?;
        return Ok(());
    }

    // Get both users
    let sender = database.get_user(discord_id).await?;
    let receiver = database.get_user(target_user).await?;

    match (sender, receiver) {
        (Some(mut sender), Some(mut receiver)) => {
            if let Err(e) = sender.deduct_coins(amount) {
                command.create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message.content(format!("❌ Transfer failed: {}", e)).ephemeral(true)
                        })
                }).await?;
                return Ok(());
            }

            receiver.add_coins(amount);
            
            database.update_user(&sender).await?;
            database.update_user(&receiver).await?;

            command.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message
                            .embed(|embed| {
                                embed
                                    .title("💸 Transfer Successful!")
                                    .description(format!("Successfully transferred **{} coins** to <@{}>!", amount, target_user))
                                    .field("Your Balance", format!("{} coins", sender.coins), true)
                                    .field("Transfer Amount", format!("{} coins", amount), true)
                                    .field("Transaction Fee", "0 coins", true)
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
                        message.content("❌ One or both users are not registered. Both users need to use `/login` first.").ephemeral(true)
                    })
            }).await?;
        }
    }

    Ok(())
}

async fn show_balance(ctx: &Context, command: &ApplicationCommandInteraction, database: &Database, discord_id: u64) -> Result<()> {
    match database.get_user(discord_id).await {
        Ok(Some(user)) => {
            command.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message
                            .embed(|embed| {
                                embed
                                    .title("💰 Your Balance")
                                    .description(format!("You have **{}** coins", user.coins))
                                    .color(0x00ff00)
                            })
                            .ephemeral(true)
                    })
            }).await?;
        }
        Ok(None) => {
            command.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message.content("You need to login first! Use `/login`").ephemeral(true)
                    })
            }).await?;
        }
        Err(e) => {
            command.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message.content(format!("Error: {}", e)).ephemeral(true)
                    })
            }).await?;
        }
    }

    Ok(())
}

async fn earn_afk(ctx: &Context, command: &ApplicationCommandInteraction, database: &Database, discord_id: u64) -> Result<()> {
    // Placeholder implementation - in real bot, verify AFK page completion
    match database.get_user(discord_id).await {
        Ok(Some(mut user)) => {
            user.add_coins(10); // Award 10 coins for AFK
            database.update_user(&user).await?;
            
            command.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message
                            .embed(|embed| {
                                embed
                                    .title("🎉 Coins Earned!")
                                    .description("You earned **10 coins** from the AFK page!\nVisit the AFK page to earn more.")
                                    .color(0x00ff00)
                            })
                            .ephemeral(true)
                    })
            }).await?;
        }
        Ok(None) => {
            command.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message.content("You need to login first! Use `/login`").ephemeral(true)
                    })
            }).await?;
        }
        Err(e) => {
            command.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message.content(format!("Error: {}", e)).ephemeral(true)
                    })
            }).await?;
        }
    }

    Ok(())
}

async fn earn_linkvertise(ctx: &Context, command: &ApplicationCommandInteraction, database: &Database, discord_id: u64) -> Result<()> {
    // Placeholder implementation - in real bot, verify Linkvertise completion
    let proof_url = command.data.options.iter()
        .find(|opt| opt.name == "proof_url")
        .and_then(|opt| opt.value.as_ref())
        .and_then(|val| val.as_str());

    if proof_url.is_none() {
        command.create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.content("Please provide a proof URL").ephemeral(true)
                })
        }).await?;
        return Ok(());
    }

    match database.get_user(discord_id).await {
        Ok(Some(mut user)) => {
            user.add_coins(25); // Award 25 coins for Linkvertise
            database.update_user(&user).await?;
            
            command.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message
                            .embed(|embed| {
                                embed
                                    .title("🎉 Coins Earned!")
                                    .description("You earned **25 coins** from Linkvertise!")
                                    .color(0x00ff00)
                            })
                            .ephemeral(true)
                    })
            }).await?;
        }
        Ok(None) => {
            command.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message.content("You need to login first! Use `/login`").ephemeral(true)
                    })
            }).await?;
        }
        Err(e) => {
            command.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message.content(format!("Error: {}", e)).ephemeral(true)
                    })
            }).await?;
        }
    }

    Ok(())
}

async fn gift_coins(ctx: &Context, command: &ApplicationCommandInteraction, database: &Database, discord_id: u64) -> Result<()> {
    let target_user = command.data.options.iter()
        .find(|opt| opt.name == "user")
        .and_then(|opt| opt.value.as_ref())
        .and_then(|val| val.as_str())
        .and_then(|s| s.parse::<u64>().ok());

    let amount = command.data.options.iter()
        .find(|opt| opt.name == "amount")
        .and_then(|opt| opt.value.as_ref())
        .and_then(|val| val.as_i64())
        .unwrap_or(0);

    if target_user.is_none() || amount <= 0 {
        command.create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.content("Please specify a valid user and amount").ephemeral(true)
                })
        }).await?;
        return Ok(());
    }

    let target_user = target_user.unwrap();
    if target_user == discord_id {
        command.create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.content("You cannot gift coins to yourself!").ephemeral(true)
                })
        }).await?;
        return Ok(());
    }

    // Get both users
    let sender = database.get_user(discord_id).await?;
    let receiver = database.get_user(target_user).await?;

    match (sender, receiver) {
        (Some(mut sender), Some(mut receiver)) => {
            if let Err(e) = sender.deduct_coins(amount) {
                command.create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message.content(format!("Error: {}", e)).ephemeral(true)
                        })
                }).await?;
                return Ok(());
            }

            receiver.add_coins(amount);
            
            database.update_user(&sender).await?;
            database.update_user(&receiver).await?;

            command.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message
                            .embed(|embed| {
                                embed
                                    .title("🎁 Gift Sent!")
                                    .description(format!("Successfully gifted **{} coins** to <@{}>!", amount, target_user))
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
                        message.content("One or both users are not registered. Both users need to use `/login` first.").ephemeral(true)
                    })
            }).await?;
        }
    }

    Ok(())
}
