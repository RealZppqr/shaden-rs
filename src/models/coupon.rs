use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coupon {
    pub code: String,
    pub coins: i64,
    pub resources: Option<super::Resources>,
    pub max_uses: Option<i64>,
    pub used_count: i64,
    pub used_by: HashSet<u64>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub created_by: u64,
}

impl Coupon {
    pub fn new(code: String, coins: i64, resources: Option<super::Resources>, max_uses: Option<i64>, expires_at: Option<DateTime<Utc>>, created_by: u64) -> Self {
        Self {
            code,
            coins,
            resources,
            max_uses,
            used_count: 0,
            used_by: HashSet::new(),
            expires_at,
            created_at: Utc::now(),
            created_by,
        }
    }

    pub fn can_be_used(&self, user_id: u64) -> bool {
        // Check if expired
        if let Some(expires_at) = self.expires_at {
            if Utc::now() > expires_at {
                return false;
            }
        }

        // Check if user already used it
        if self.used_by.contains(&user_id) {
            return false;
        }

        // Check usage limit
        if let Some(max_uses) = self.max_uses {
            if self.used_count >= max_uses {
                return false;
            }
        }

        true
    }

    pub fn use_coupon(&mut self, user_id: u64) -> bool {
        if !self.can_be_used(user_id) {
            return false;
        }

        self.used_by.insert(user_id);
        self.used_count += 1;
        true
    }
}
