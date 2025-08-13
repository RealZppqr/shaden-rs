use anyhow::Result;
use serenity::prelude::*;
use serenity::all::{Command, CommandOptionType, ApplicationCommandInteraction};
use crate::config::Config;

pub mod coins;
pub mod coupons;
pub mod servers;
pub mod store;
pub mod admin;
pub mod join_rewards;

pub async fn register_commands(ctx: &Context, config: &Config) -> Result<()> {
    // Register all slash commands
    Command::create_global_command(&ctx.http, |command| {
        command.name("login").description("Register or login to the bot")
    }).await?;

    Command::create_global_command(&ctx.http, |command| {
        command
            .name("coins")
            .description("Manage your coins")
            .create_option(|option| {
                option
                    .name("action")
                    .description("Action to perform")
                    .kind(CommandOptionType::String)
                    .required(true)
                    .add_string_choice("balance", "balance")
                    .add_string_choice("earn-afk", "earn-afk")
                    .add_string_choice("earn-linkvertise", "earn-linkvertise")
                    .add_string_choice("gift", "gift")
            })
            .create_option(|option| {
                option
                    .name("user")
                    .description("User to gift coins to")
                    .kind(CommandOptionType::User)
                    .required(false)
            })
            .create_option(|option| {
                option
                    .name("amount")
                    .description("Amount of coins")
                    .kind(CommandOptionType::Integer)
                    .required(false)
            })
            .create_option(|option| {
                option
                    .name("proof_url")
                    .description("Proof URL for linkvertise")
                    .kind(CommandOptionType::String)
                    .required(false)
            })
    }).await?;

    if config.enable_transfer {
        Command::create_global_command(&ctx.http, |command| {
            command
                .name("transfer")
                .description("Transfer coins to another user")
                .create_option(|option| {
                    option
                        .name("user")
                        .description("User to transfer coins to")
                        .kind(CommandOptionType::User)
                        .required(true)
                })
                .create_option(|option| {
                    option
                        .name("amount")
                        .description("Amount of coins to transfer")
                        .kind(CommandOptionType::Integer)
                        .required(true)
                        .min_int_value(1)
                })
        }).await?;
    }

    Command::create_global_command(&ctx.http, |command| {
        command
            .name("servers")
            .description("Manage your servers")
            .create_option(|option| {
                option
                    .name("action")
                    .description("Action to perform")
                    .kind(CommandOptionType::String)
                    .required(true)
                    .add_string_choice("list", "list")
                    .add_string_choice("create", "create")
                    .add_string_choice("view", "view")
                    .add_string_choice("power", "power")
            })
            .create_option(|option| {
                option
                    .name("server_id")
                    .description("Server ID")
                    .kind(CommandOptionType::String)
                    .required(false)
            })
            .create_option(|option| {
                option
                    .name("plan")
                    .description("Server plan")
                    .kind(CommandOptionType::String)
                    .required(false)
            })
            .create_option(|option| {
                option
                    .name("name")
                    .description("Server name")
                    .kind(CommandOptionType::String)
                    .required(false)
            })
    }).await?;

    if config.enable_delete {
        Command::create_global_command(&ctx.http, |command| {
            command
                .name("delete")
                .description("Delete a server")
                .create_option(|option| {
                    option
                        .name("server_id")
                        .description("Server ID to delete")
                        .kind(CommandOptionType::String)
                        .required(true)
                })
        }).await?;
    }

    if config.enable_renew {
        Command::create_global_command(&ctx.http, |command| {
            command
                .name("renew")
                .description("Renew a server to prevent suspension")
                .create_option(|option| {
                    option
                        .name("server_id")
                        .description("Server ID to renew")
                        .kind(CommandOptionType::String)
                        .required(true)
                })
                .create_option(|option| {
                    option
                        .name("duration")
                        .description("Renewal duration in days")
                        .kind(CommandOptionType::Integer)
                        .required(false)
                        .min_int_value(1)
                        .max_int_value(365)
                })
        }).await?;
    }

    Command::create_global_command(&ctx.http, |command| {
        command
            .name("store")
            .description("Browse and purchase items")
            .create_option(|option| {
                option
                    .name("action")
                    .description("Action to perform")
                    .kind(CommandOptionType::String)
                    .required(true)
                    .add_string_choice("list", "list")
                    .add_string_choice("buy", "buy")
                    .add_string_choice("categories", "categories")
            })
            .create_option(|option| {
                option
                    .name("item_id")
                    .description("Item ID to purchase")
                    .kind(CommandOptionType::String)
                    .required(false)
            })
            .create_option(|option| {
                option
                    .name("category")
                    .description("Filter by category")
                    .kind(CommandOptionType::String)
                    .required(false)
            })
    }).await?;

    // Add more command registrations here...
    
    Ok(())
}

pub async fn help(ctx: &Context, command: &ApplicationCommandInteraction) -> Result<()> {
    command.create_interaction_response(&ctx.http, |response| {
        response
            .kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| {
                message
                    .embed(|embed| {
                        embed
                            .title("Shaden-RS Bot Help")
                            .description("Available commands:")
                            .field("/login", "Register or login to the bot", false)
                            .field("/coins balance", "Check your coin balance", false)
                            .field("/coins earn afk", "Earn coins via AFK page", false)
                            .field("/coins gift <user> <amount>", "Gift coins to another user", false)
                            .field("/transfer <user> <amount>", "Transfer coins to another user", false)
                            .field("/servers list", "List your servers", false)
                            .field("/servers create <plan> <name>", "Create a new server", false)
                            .field("/delete <server_id>", "Delete a server", false)
                            .field("/renew <server_id> [duration]", "Renew a server", false)
                            .field("/store list", "View available items", false)
                            .color(0x00ff00)
                    })
                    .ephemeral(true)
            })
    }).await?;

    Ok(())
}
