// Temporarily disabled due to dependency issues
use crate::config::Config;
use crate::errors::{BotError, BotResult};

#[derive(Debug, Clone)]
pub struct StripeService {
    client: (), // Temporarily disabled
}

impl StripeService {
    pub fn new(_config: &Config) -> Self {
        // Temporarily disabled
        Self { client: () }
    }

    pub async fn create_checkout_session(&self, _amount_cents: i64, _coins: i64, _discord_id: u64) -> BotResult<()> {
        // Temporarily disabled
        Err(BotError::Stripe("Stripe service temporarily disabled".to_string()))
    }

    pub async fn get_session(&self, _session_id: &str) -> BotResult<()> {
        // Temporarily disabled
        Err(BotError::Stripe("Stripe service temporarily disabled".to_string()))
    }
}
