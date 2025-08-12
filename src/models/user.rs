use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub discord_id: u64,
    pub coins: i64,
    pub resources: Resources,
    pub pterodactyl_api_key: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Resources {
    pub ram: i64,      // MB
    pub cpu: i64,      // %
    pub disk: i64,     // MB
    pub databases: i64,
    pub allocations: i64,
    pub backups: i64,
}

impl User {
    pub fn new(discord_id: u64) -> Self {
        let now = Utc::now();
        Self {
            discord_id,
            coins: 0,
            resources: Resources::default(),
            pterodactyl_api_key: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn can_afford(&self, cost: i64) -> bool {
        self.coins >= cost
    }

    pub fn deduct_coins(&mut self, amount: i64) -> Result<(), crate::errors::BotError> {
        if !self.can_afford(amount) {
            return Err(crate::errors::BotError::InsufficientCoins {
                needed: amount,
                available: self.coins,
            });
        }
        self.coins -= amount;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn add_coins(&mut self, amount: i64) {
        self.coins += amount;
        self.updated_at = Utc::now();
    }
}
