use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: Uuid,
    pub discord_id: u64,
    pub stripe_session_id: String,
    pub amount_cents: i64,
    pub coins: i64,
    pub status: OrderStatus,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending,
    Completed,
    Failed,
    Cancelled,
}

impl Order {
    pub fn new(discord_id: u64, stripe_session_id: String, amount_cents: i64, coins: i64) -> Self {
        Self {
            id: Uuid::new_v4(),
            discord_id,
            stripe_session_id,
            amount_cents,
            coins,
            status: OrderStatus::Pending,
            created_at: Utc::now(),
            completed_at: None,
        }
    }

    pub fn complete(&mut self) {
        self.status = OrderStatus::Completed;
        self.completed_at = Some(Utc::now());
    }
}
