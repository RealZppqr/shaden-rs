use anyhow::Result;
use serenity::prelude::*;
use serenity::all::{ApplicationCommandInteraction, InteractionResponseType};
use crate::services::Database;
use crate::config::Config;

pub async fn handle_store(ctx: &Context, command: &ApplicationCommandInteraction, database: &Database, config: &Config) -> Result<()> {
    let action = command.data.options.get(0)
        .and_then(|opt| opt.value.as_ref())
        .and_then(|val| val.as_str())
        .unwrap_or("list");

    match action {
        "list" => list_store_items(ctx, command, config).await,
        "buy" => buy_store_item(ctx, command, database, config).await,
        "categories" => list_categories(ctx, command, config).await,
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

async fn list_store_items(ctx: &Context, command: &ApplicationCommandInteraction, config: &Config) -> Result<()> {
    let category_filter = command.data.options.iter()
        .find(|opt| opt.name == "category")
        .and_then(|opt| opt.value.as_ref())
        .and_then(|val| val.as_str());

    let items: Vec<_> = config.store_config.items.iter()
        .filter(|item| item.enabled)
        .filter(|item| {
            if let Some(filter) = category_filter {
                item.category.eq_ignore_ascii_case(filter)
            } else {
                true
            }
        })
        .collect();

    if items.is_empty() {
        let message = if category_filter.is_some() {
            format!("No items found in category '{}'", category_filter.unwrap())
        } else {
            "No items available in the store".to_string()
        };

        command.create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message_builder| {
                    message_builder.content(message).ephemeral(true)
                })
        }).await?;
        return Ok(());
    }

    let title = if let Some(filter) = category_filter {
        format!("🛒 Store - {} Category", filter)
    } else {
        "🛒 Store Items".to_string()
    };

    command.create_interaction_response(&ctx.http, |response| {
        response
            .kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| {
                message
                    .embed(|embed| {
                        let mut embed = embed
                            .title(&title)
                            .description("Available items for purchase:")
                            .color(0x00ff00);

                        for item in items.iter().take(25) { // Discord embed field limit
                            let duration_text = if let Some(days) = item.duration_days {
                                format!(" ({} days)", days)
                            } else {
                                " (permanent)".to_string()
                            };

                            let field_value = format!("{}\n💰 {} coins{}\n🏷️ ID: `{}`", 
                                item.description, item.price, duration_text, item.id);

                            embed = embed.field(&item.name, field_value, false);
                        }

                        embed.footer(|f| f.text("Use /store buy <item_id> to purchase • /store categories to see all categories"))
                    })
                    .ephemeral(true)
            })
    }).await?;

    Ok(())
}

async fn list_categories(ctx: &Context, command: &ApplicationCommandInteraction, config: &Config) -> Result<()> {
    let mut category_counts = std::collections::HashMap::new();
    
    for item in &config.store_config.items {
        if item.enabled {
            *category_counts.entry(&item.category).or_insert(0) += 1;
        }
    }

    command.create_interaction_response(&ctx.http, |response| {
        response
            .kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| {
                message
                    .embed(|embed| {
                        let mut embed = embed
                            .title("🏷️ Store Categories")
                            .description("Browse items by category:")
                            .color(0x00ff00);

                        for category in &config.store_config.categories {
                            let count = category_counts.get(category).unwrap_or(&0);
                            embed = embed.field(category, format!("{} items available", count), true);
                        }

                        embed.footer(|f| f.text("Use /store list category:<name> to filter by category"))
                    })
                    .ephemeral(true)
            })
    }).await?;

    Ok(())
}

async fn buy_store_item(ctx: &Context, command: &ApplicationCommandInteraction, database: &Database, config: &Config) -> Result<()> {
    let discord_id = command.user.id.0;
    let item_id = command.data.options.iter()
        .find(|opt| opt.name == "item_id")
        .and_then(|opt| opt.value.as_ref())
        .and_then(|val| val.as_str());

    if item_id.is_none() {
        command.create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.content("❌ Please provide an item ID. Use `/store list` to see available items.").ephemeral(true)
                })
        }).await?;
        return Ok(());
    }

    let item_id = item_id.unwrap();

    // Find the item in config
    let store_item = config.store_config.items.iter()
        .find(|item| item.id == item_id && item.enabled);

    if store_item.is_none() {
        command.create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.content("❌ Invalid item ID or item is disabled. Use `/store list` to see available items.").ephemeral(true)
                })
        }).await?;
        return Ok(());
    }

    let store_item = store_item.unwrap();

    // Get user
    let user = database.get_user(discord_id).await?;
    if user.is_none() {
        command.create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.content("❌ You need to login first! Use `/login`").ephemeral(true)
                })
        }).await?;
        return Ok(());
    }

    let mut user = user.unwrap();

    // Check if user can afford it
    if let Err(e) = user.deduct_coins(store_item.price as i64) {
        command.create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.content(format!("❌ Purchase failed: {}", e)).ephemeral(true)
                })
        }).await?;
        return Ok(());
    }

    // Apply the resource update if available
    if let Some(resources) = &store_item.resources {
        user.resources.ram += resources.ram as i64;
        user.resources.cpu += resources.cpu as i64;
        user.resources.disk += resources.disk as i64;
        user.resources.databases += resources.databases as i64;
        user.resources.allocations += resources.allocations as i64;
        user.resources.backups += resources.backups as i64;
    }

    // Update user in database
    database.update_user(&user).await?;

    let duration_text = if let Some(days) = store_item.duration_days {
        format!(" for {} days", days)
    } else {
        " permanently".to_string()
    };

    let resource_text = if let Some(resources) = &store_item.resources {
        let mut parts = Vec::new();
        if resources.ram > 0 { parts.push(format!("{}MB RAM", resources.ram)); }
        if resources.cpu > 0 { parts.push(format!("{}% CPU", resources.cpu)); }
        if resources.disk > 0 { parts.push(format!("{}MB Disk", resources.disk)); }
        if resources.databases > 0 { parts.push(format!("{} Database(s)", resources.databases)); }
        if resources.allocations > 0 { parts.push(format!("{} Allocation(s)", resources.allocations)); }
        if resources.backups > 0 { parts.push(format!("{} Backup(s)", resources.backups)); }
        
        if parts.is_empty() {
            "No resources".to_string()
        } else {
            parts.join(", ")
        }
    } else {
        "No resources".to_string()
    };

    command.create_interaction_response(&ctx.http, |response| {
        response
            .kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| {
                message
                    .embed(|embed| {
                        embed
                            .title("🛒 Purchase Successful!")
                            .description(format!("Successfully purchased **{}**{}", store_item.name, duration_text))
                            .field("Cost", format!("{} coins", store_item.price), true)
                            .field("Resources Added", resource_text, true)
                            .field("Remaining Coins", format!("{} coins", user.coins), true)
                            .color(0x00ff00)
                    })
                    .ephemeral(true)
            })
    }).await?;

    Ok(())
}
