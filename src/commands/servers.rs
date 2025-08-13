use anyhow::Result;
use serenity::prelude::*;
use serenity::all::{ApplicationCommandInteraction, InteractionResponseType};
use crate::services::{Database, QueueService, QueueJob, JobType};
use crate::models::{Server, Resources, ServerStatus};
use crate::config::Config;

pub async fn handle_servers(ctx: &Context, command: &ApplicationCommandInteraction, database: &Database, config: &Config) -> Result<()> {
    let discord_id = command.user.id.0;
    let action = command.data.options.get(0)
        .and_then(|opt| opt.value.as_ref())
        .and_then(|val| val.as_str())
        .unwrap_or("list");

    match action {
        "list" => list_servers(ctx, command, database, discord_id).await,
        "create" => create_server(ctx, command, database, discord_id).await,
        "view" => view_server(ctx, command, database, discord_id).await,
        "delete" => delete_server_command(ctx, command, database, config).await,
        "power" => power_server(ctx, command, database, discord_id).await,
        "renew" => renew_server(ctx, command, database, config).await,
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

async fn list_servers(ctx: &Context, command: &ApplicationCommandInteraction, database: &Database, discord_id: u64) -> Result<()> {
    match database.get_user_servers(discord_id).await {
        Ok(servers) => {
            if servers.is_empty() {
                command.create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message
                                .embed(|embed| {
                                    embed
                                        .title("🖥️ Your Servers")
                                        .description("You don't have any servers yet.\nUse `/servers create` to create one!")
                                        .color(0xffaa00)
                                })
                                .ephemeral(true)
                        })
                }).await?;
            } else {
                let mut description = String::new();
                for server in servers {
                    let status_emoji = match server.status {
                        ServerStatus::Running => "🟢",
                        ServerStatus::Stopped => "🔴",
                        ServerStatus::Creating => "🟡",
                        ServerStatus::Suspended => "🟠",
                        ServerStatus::Deleted => "⚫",
                    };
                    
                    description.push_str(&format!(
                        "{} **{}** ({})\n📊 {}MB RAM, {}% CPU, {}MB Disk\n⏰ Expires: {}\n\n",
                        status_emoji,
                        server.name,
                        server.id,
                        server.resources.ram,
                        server.resources.cpu,
                        server.resources.disk,
                        server.expires_at.format("%Y-%m-%d %H:%M UTC")
                    ));
                }

                command.create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message
                                .embed(|embed| {
                                    embed
                                        .title("🖥️ Your Servers")
                                        .description(description)
                                        .color(0x00ff00)
                                })
                                .ephemeral(true)
                        })
                }).await?;
            }
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

async fn create_server(ctx: &Context, command: &ApplicationCommandInteraction, database: &Database, discord_id: u64) -> Result<()> {
    let plan = command.data.options.iter()
        .find(|opt| opt.name == "plan")
        .and_then(|opt| opt.value.as_ref())
        .and_then(|val| val.as_str());

    let name = command.data.options.iter()
        .find(|opt| opt.name == "name")
        .and_then(|opt| opt.value.as_ref())
        .and_then(|val| val.as_str());

    if plan.is_none() || name.is_none() {
        command.create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.content("Please provide both plan and name").ephemeral(true)
                })
        }).await?;
        return Ok(());
    }

    let plan = plan.unwrap();
    let name = name.unwrap();

    // Get user to check resources
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

    let user = user.unwrap();

    // Define plan resources (this should be configurable)
    let plan_resources = match plan {
        "free" => Resources {
            ram: 512,
            cpu: 50,
            disk: 1024,
            databases: 1,
            allocations: 1,
            backups: 1,
        },
        "basic" => Resources {
            ram: 1024,
            cpu: 100,
            disk: 2048,
            databases: 2,
            allocations: 2,
            backups: 2,
        },
        _ => {
            command.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message.content("Invalid plan. Available plans: free, basic").ephemeral(true)
                    })
            }).await?;
            return Ok(());
        }
    };

    // Check if user has enough resources
    if user.resources.ram < plan_resources.ram ||
       user.resources.cpu < plan_resources.cpu ||
       user.resources.disk < plan_resources.disk {
        command.create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.content("Insufficient resources for this plan").ephemeral(true)
                })
        }).await?;
        return Ok(());
    }

    // Create server and add to queue
    let server = Server::new(discord_id, name.to_string(), plan.to_string(), plan_resources);
    database.create_server(&server).await?;

    let queue_service = QueueService::new(database.clone());
    let job = QueueJob::new(
        JobType::CreateServer,
        discord_id,
        serde_json::json!({ "server_id": server.id.to_string() })
    );
    queue_service.enqueue(job).await?;

    command.create_interaction_response(&ctx.http, |response| {
        response
            .kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| {
                message
                    .embed(|embed| {
                        embed
                            .title("🚀 Server Creation Started")
                            .description(format!("Server **{}** has been queued for creation!\n\nServer ID: `{}`\nPlan: {}\n\nYour server will be created shortly.", name, server.id, plan))
                            .color(0x00ff00)
                    })
                    .ephemeral(true)
            })
    }).await?;

    Ok(())
}

async fn view_server(ctx: &Context, command: &ApplicationCommandInteraction, database: &Database, discord_id: u64) -> Result<()> {
    let server_id = command.data.options.iter()
        .find(|opt| opt.name == "server_id")
        .and_then(|opt| opt.value.as_ref())
        .and_then(|val| val.as_str());

    if server_id.is_none() {
        command.create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.content("Please provide a server ID").ephemeral(true)
                })
        }).await?;
        return Ok(());
    }

    let server_id = server_id.unwrap();

    match database.get_server(server_id).await {
        Ok(Some(server)) => {
            if server.discord_id != discord_id {
                command.create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message.content("You don't own this server").ephemeral(true)
                        })
                }).await?;
                return Ok(());
            }

            let status_emoji = match server.status {
                ServerStatus::Running => "🟢 Running",
                ServerStatus::Stopped => "🔴 Stopped",
                ServerStatus::Creating => "🟡 Creating",
                ServerStatus::Suspended => "🟠 Suspended",
                ServerStatus::Deleted => "⚫ Deleted",
            };

            command.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message
                            .embed(|embed| {
                                embed
                                    .title(format!("🖥️ Server: {}", server.name))
                                    .field("Status", status_emoji, true)
                                    .field("Plan", &server.plan, true)
                                    .field("Server ID", server.id.to_string(), true)
                                    .field("Resources", format!("🖥️ {}MB RAM\n⚡ {}% CPU\n💾 {}MB Disk", server.resources.ram, server.resources.cpu, server.resources.disk), true)
                                    .field("Limits", format!("🗄️ {} Databases\n🌐 {} Allocations\n💾 {} Backups", server.resources.databases, server.resources.allocations, server.resources.backups), true)
                                    .field("Expires", server.expires_at.format("%Y-%m-%d %H:%M UTC").to_string(), true)
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
                        message.content("Server not found").ephemeral(true)
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

pub async fn delete_server_command(ctx: &Context, command: &ApplicationCommandInteraction, database: &Database, config: &Config) -> Result<()> {
    if !config.enable_delete {
        command.create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.content("❌ Server deletion is currently disabled by administrators.").ephemeral(true)
                })
        }).await?;
        return Ok(());
    }

    let discord_id = command.user.id.0;
    
    let server_id = command.data.options.iter()
        .find(|opt| opt.name == "server_id")
        .and_then(|opt| opt.value.as_ref())
        .and_then(|val| val.as_str());

    if server_id.is_none() {
        command.create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.content("❌ Please provide a server ID to delete.").ephemeral(true)
                })
        }).await?;
        return Ok(());
    }

    let server_id = server_id.unwrap();

    // Get server and verify ownership
    match database.get_server(server_id).await {
        Ok(Some(server)) => {
            if server.discord_id != discord_id {
                command.create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message.content("❌ You don't own this server.").ephemeral(true)
                        })
                }).await?;
                return Ok(());
            }

            // Perform deletion
            match database.delete_server(server_id).await {
                Ok(_) => {
                    // Add deletion job to queue
                    let queue_service = QueueService::new(database.clone());
                    let job = QueueJob::new(
                        JobType::DeleteServer,
                        discord_id,
                        serde_json::json!({ "server_id": server_id })
                    );
                    queue_service.enqueue(job).await?;

                    command.create_interaction_response(&ctx.http, |response| {
                        response
                            .kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|message| {
                                message
                                    .embed(|embed| {
                                        embed
                                            .title("🗑️ Server Deletion Started")
                                            .description(format!("Server **{}** has been queued for deletion!\n\n⚠️ **This action cannot be undone!**\n\nThe server will be permanently deleted from Pterodactyl shortly.", server.name))
                                            .field("Server ID", server_id, true)
                                            .field("Status", "Queued for deletion", true)
                                            .color(0xff6b6b)
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
                                message.content(format!("❌ Deletion failed: {}", e)).ephemeral(true)
                            })
                    }).await?;
                }
            }
        }
        Ok(None) => {
            command.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message.content("❌ Server not found.").ephemeral(true)
                    })
            }).await?;
        }
        Err(e) => {
            command.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message.content(format!("❌ Error: {}", e)).ephemeral(true)
                    })
            }).await?;
        }
    }

    Ok(())
}

async fn power_server(ctx: &Context, command: &ApplicationCommandInteraction, database: &Database, discord_id: u64) -> Result<()> {
    // Implementation for server power actions (start/stop/restart)
    command.create_interaction_response(&ctx.http, |response| {
        response
            .kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| {
                message.content("Server power actions not yet implemented").ephemeral(true)
            })
    }).await?;

    Ok(())
}

pub async fn renew_server(ctx: &Context, command: &ApplicationCommandInteraction, database: &Database, config: &Config) -> Result<()> {
    if !config.enable_renew {
        command.create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.content("❌ Server renewal is currently disabled by administrators.").ephemeral(true)
                })
        }).await?;
        return Ok(());
    }

    let discord_id = command.user.id.0;
    
    let server_id = command.data.options.iter()
        .find(|opt| opt.name == "server_id")
        .and_then(|opt| opt.value.as_ref())
        .and_then(|val| val.as_str());

    let duration = command.data.options.iter()
        .find(|opt| opt.name == "duration")
        .and_then(|opt| opt.value.as_ref())
        .and_then(|val| val.as_i64())
        .unwrap_or(30) as u32; // Default 30 days

    if server_id.is_none() {
        command.create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.content("❌ Please provide a server ID to renew.").ephemeral(true)
                })
        }).await?;
        return Ok(());
    }

    let server_id = server_id.unwrap();

    // Get server and verify ownership
    match database.get_server(server_id).await {
        Ok(Some(server)) => {
            if server.discord_id != discord_id {
                command.create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message.content("❌ You don't own this server.").ephemeral(true)
                        })
                }).await?;
                return Ok(());
            }

            // Calculate renewal cost (1 coin per day)
            let renewal_cost = duration as u64;

            // Get user to check coins
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

            let user = user.unwrap();

            if user.coins < renewal_cost {
                command.create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message.content(format!("❌ Insufficient coins! Renewal costs {} coins ({} days × 1 coin/day). You have {} coins.", renewal_cost, duration, user.coins)).ephemeral(true)
                        })
                }).await?;
                return Ok(());
            }

            // Perform renewal
            match database.renew_server(server_id, duration, renewal_cost).await {
                Ok(_) => {
                    // Get updated server to get new expiry date
                    let updated_server = database.get_server(server_id).await?.unwrap();
                    command.create_interaction_response(&ctx.http, |response| {
                        response
                            .kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|message| {
                                message
                                    .embed(|embed| {
                                        embed
                                            .title("🔄 Server Renewed Successfully!")
                                            .description(format!("Server **{}** has been renewed for {} days!", server.name, duration))
                                            .field("Cost", format!("{} coins", renewal_cost), true)
                                            .field("New Expiry Date", updated_server.expires_at.format("%Y-%m-%d %H:%M UTC").to_string(), true)
                                            .field("Remaining Coins", format!("{} coins", user.coins - renewal_cost), true)
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
                                message.content(format!("❌ Renewal failed: {}", e)).ephemeral(true)
                            })
                    }).await?;
                }
            }
        }
        Ok(None) => {
            command.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message.content("❌ Server not found.").ephemeral(true)
                    })
            }).await?;
        }
        Err(e) => {
            command.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message.content(format!("❌ Error: {}", e)).ephemeral(true)
                    })
            }).await?;
        }
    }

    Ok(())
}
