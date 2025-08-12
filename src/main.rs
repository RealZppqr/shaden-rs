use anyhow::Result;
use serenity::prelude::*;
use serenity::framework::standard::StandardFramework;
use serenity::model::gateway::Ready;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::application::command::Command;
use tracing::{info, error};

mod config;
mod models;
mod services;
mod commands;
mod errors;

use config::Config;
use services::db::Database;

struct Handler {
    database: Database,
    config: Config,
}

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
        
        // Register slash commands
        if let Err(e) = commands::register_commands(&ctx, &self.config).await {
            error!("Failed to register commands: {}", e);
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let result = match command.data.name.as_str() {
                "login" => commands::coins::login(&ctx, &command, &self.database).await,
                "coins" => commands::coins::handle_coins(&ctx, &command, &self.database).await,
                "coupons" => commands::coupons::handle_coupons(&ctx, &command, &self.database).await,
                "servers" => commands::servers::handle_servers(&ctx, &command, &self.database, &self.config).await,
                "store" => commands::store::handle_store(&ctx, &command, &self.database, &self.config).await,
                "renew" => commands::servers::renew_server(&ctx, &command, &self.database, &self.config).await,
                "transfer" => commands::coins::transfer_coins(&ctx, &command, &self.database, &self.config).await,
                "delete" => commands::servers::delete_server_command(&ctx, &command, &self.database, &self.config).await,
                "join-for-reward" => commands::join_rewards::handle_join_rewards(&ctx, &command, &self.database).await,
                "admin" => commands::admin::handle_admin(&ctx, &command, &self.database, &self.config).await,
                "help" => commands::help(&ctx, &command).await,
                _ => {
                    command.create_interaction_response(&ctx.http, |response| {
                        response
                            .kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|message| {
                                message.content("Unknown command").ephemeral(true)
                            })
                    }).await
                }
            };

            if let Err(e) = result {
                error!("Error handling command {}: {}", command.data.name, e);
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let config = Config::load()?;
    let database = Database::new(&config).await?;

    let mut client = Client::builder(&config.discord_token, GatewayIntents::empty())
        .event_handler(Handler { database, config })
        .await?;

    info!("Starting shaden-rs bot...");
    client.start().await?;

    Ok(())
}
