use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub id: Uuid,
    pub discord_id: u64,
    pub pterodactyl_id: Option<i64>,
    pub name: String,
    pub plan: String,
    pub resources: super::Resources,
    pub status: ServerStatus,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerStatus {
    Creating,
    Running,
    Stopped,
    Suspended,
    Deleted,
}

impl Server {
    pub fn new(discord_id: u64, name: String, plan: String, resources: super::Resources) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            discord_id,
            pterodactyl_id: None,
            name,
            plan,
            resources,
            status: ServerStatus::Creating,
            expires_at: now + chrono::Duration::days(30), // Default 30 days
            created_at: now,
            updated_at: now,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn extend_expiry(&mut self, days: i64) {
        self.expires_at = self.expires_at + chrono::Duration::days(days);
        self.updated_at = Utc::now();
    }
}
