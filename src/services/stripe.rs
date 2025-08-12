use anyhow::Result;
use stripe::{Client, CheckoutSession, CheckoutSessionMode, CreateCheckoutSession, CreateCheckoutSessionLineItems, CreateCheckoutSessionLineItemsPriceData, Currency};
use crate::config::Config;
use crate::errors::{BotError, BotResult};

#[derive(Debug, Clone)]
pub struct StripeService {
    client: Client,
}

impl StripeService {
    pub fn new(config: &Config) -> Self {
        let client = Client::new(config.stripe_secret_key.clone());
        Self { client }
    }

    pub async fn create_checkout_session(&self, amount_cents: i64, coins: i64, discord_id: u64) -> BotResult<CheckoutSession> {
        let mut params = CreateCheckoutSession::new();
        params.mode = Some(CheckoutSessionMode::Payment);
        params.success_url = Some("https://discord.com/channels/@me".to_string());
        params.cancel_url = Some("https://discord.com/channels/@me".to_string());
        params.metadata = Some(
            [("discord_id".to_string(), discord_id.to_string()), ("coins".to_string(), coins.to_string())]
                .iter()
                .cloned()
                .collect()
        );

        let mut line_item = CreateCheckoutSessionLineItems::default();
        line_item.quantity = Some(1);
        
        let mut price_data = CreateCheckoutSessionLineItemsPriceData::default();
        price_data.currency = Currency::USD;
        price_data.unit_amount = Some(amount_cents);
        price_data.product_data = Some(stripe::CreateCheckoutSessionLineItemsPriceDataProductData {
            name: format!("{} Coins", coins),
            description: Some(format!("Purchase {} coins for your account", coins)),
            ..Default::default()
        });
        
        line_item.price_data = Some(price_data);
        params.line_items = Some(vec![line_item]);

        let session = CheckoutSession::create(&self.client, params)
            .await
            .map_err(|e| BotError::Stripe(e.to_string()))?;

        Ok(session)
    }

    pub async fn get_session(&self, session_id: &str) -> BotResult<CheckoutSession> {
        let session = CheckoutSession::retrieve(&self.client, &session_id.parse().unwrap(), &[])
            .await
            .map_err(|e| BotError::Stripe(e.to_string()))?;

        Ok(session)
    }
}
