use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Config {
    pub discord_token: String,
    pub discord_app_id: u64,
    pub pterodactyl_url: String,
    pub pterodactyl_api_key: String,
    pub mongodb_uri: String,
    pub redis_uri: String,
    pub stripe_secret_key: String,
    pub stripe_public_key: String,
    pub afk_page_url: String,
    pub linkvertise_verify_url: String,
    pub admin_discord_ids: Vec<u64>,
    pub enable_transfer: bool,
    pub enable_renew: bool,
    pub enable_delete: bool,
    pub store_config: StoreConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreConfig {
    pub items: Vec<StoreItem>,
    pub categories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub price: u64, // in coins
    pub category: String,
    pub resources: Option<StoreResources>,
    pub duration_days: Option<u32>, // for temporary items
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreResources {
    pub ram: u32,
    pub cpu: u32,
    pub disk: u32,
    pub databases: u32,
    pub allocations: u32,
    pub backups: u32,
}

impl Default for StoreConfig {
    fn default() -> Self {
        Self {
            categories: vec![
                "Resources".to_string(),
                "Servers".to_string(),
                "Addons".to_string(),
            ],
            items: vec![
                StoreItem {
                    id: "ram_512".to_string(),
                    name: "512MB RAM".to_string(),
                    description: "Add 512MB RAM to your account".to_string(),
                    price: 100,
                    category: "Resources".to_string(),
                    resources: Some(StoreResources {
                        ram: 512,
                        cpu: 0,
                        disk: 0,
                        databases: 0,
                        allocations: 0,
                        backups: 0,
                    }),
                    duration_days: None,
                    enabled: true,
                },
                StoreItem {
                    id: "basic_server".to_string(),
                    name: "Basic Server Plan".to_string(),
                    description: "1GB RAM, 100% CPU, 2GB Disk".to_string(),
                    price: 500,
                    category: "Servers".to_string(),
                    resources: Some(StoreResources {
                        ram: 1024,
                        cpu: 100,
                        disk: 2048,
                        databases: 2,
                        allocations: 2,
                        backups: 2,
                    }),
                    duration_days: Some(30),
                    enabled: true,
                },
            ],
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        dotenvy::dotenv().ok();

        let admin_ids_str = env::var("ADMIN_DISCORD_IDS").unwrap_or_default();
        let admin_discord_ids = admin_ids_str
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();

        let store_config = Self::load_store_config()?;

        Ok(Config {
            discord_token: env::var("DISCORD_TOKEN").context("DISCORD_TOKEN not set")?,
            discord_app_id: env::var("DISCORD_APP_ID")
                .context("DISCORD_APP_ID not set")?
                .parse()
                .context("Invalid DISCORD_APP_ID")?,
            pterodactyl_url: env::var("PTERODACTYL_URL").context("PTERODACTYL_URL not set")?,
            pterodactyl_api_key: env::var("PTERODACTYL_API_KEY").context("PTERODACTYL_API_KEY not set")?,
            mongodb_uri: env::var("MONGODB_URI").context("MONGODB_URI not set")?,
            redis_uri: env::var("REDIS_URI").context("REDIS_URI not set")?,
            stripe_secret_key: env::var("STRIPE_SECRET_KEY").context("STRIPE_SECRET_KEY not set")?,
            stripe_public_key: env::var("STRIPE_PUBLIC_KEY").context("STRIPE_PUBLIC_KEY not set")?,
            afk_page_url: env::var("AFK_PAGE_URL").unwrap_or_else(|_| "https://example.com/afk".to_string()),
            linkvertise_verify_url: env::var("LINKVERTISE_VERIFY_URL").unwrap_or_else(|_| "https://example.com/verify".to_string()),
            admin_discord_ids,
            enable_transfer: env::var("ENABLE_TRANSFER").unwrap_or_else(|_| "true".to_string()).parse().unwrap_or(true),
            enable_renew: env::var("ENABLE_RENEW").unwrap_or_else(|_| "true".to_string()).parse().unwrap_or(true),
            enable_delete: env::var("ENABLE_DELETE").unwrap_or_else(|_| "true".to_string()).parse().unwrap_or(true),
            store_config,
        })
    }

    fn load_store_config() -> Result<StoreConfig> {
        let config_path = env::var("STORE_CONFIG_PATH").unwrap_or_else(|_| "store_config.json".to_string());
        
        if Path::new(&config_path).exists() {
            let content = fs::read_to_string(&config_path)
                .context("Failed to read store config file")?;
            let config: StoreConfig = serde_json::from_str(&content)
                .context("Failed to parse store config JSON")?;
            Ok(config)
        } else {
            // Create default config file
            let default_config = StoreConfig::default();
            let json = serde_json::to_string_pretty(&default_config)
                .context("Failed to serialize default store config")?;
            fs::write(&config_path, json)
                .context("Failed to write default store config file")?;
            Ok(default_config)
        }
    }

    pub fn is_admin(&self, user_id: u64) -> bool {
        self.admin_discord_ids.contains(&user_id)
    }
}
