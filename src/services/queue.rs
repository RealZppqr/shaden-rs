use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::errors::{BotError, BotResult};
use crate::services::Database;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueJob {
    pub id: Uuid,
    pub job_type: JobType,
    pub discord_id: u64,
    pub data: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobType {
    CreateServer,
    DeleteServer,
    Login,
}

impl QueueJob {
    pub fn new(job_type: JobType, discord_id: u64, data: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            job_type,
            discord_id,
            data,
            created_at: chrono::Utc::now(),
        }
    }
}

pub struct QueueService {
    database: Database,
}

impl QueueService {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn enqueue(&self, job: QueueJob) -> BotResult<()> {
        let mut conn = self.database.get_redis_connection().await?;
        let job_json = serde_json::to_string(&job)
            .map_err(|e| BotError::InvalidInput(format!("Failed to serialize job: {}", e)))?;
        
        conn.lpush("job_queue", job_json).await?;
        Ok(())
    }

    pub async fn dequeue(&self) -> BotResult<Option<QueueJob>> {
        let mut conn = self.database.get_redis_connection().await?;
        let job_json: Option<String> = conn.brpop("job_queue", 1.0).await?;
        
        if let Some(json) = job_json {
            let job: QueueJob = serde_json::from_str(&json)
                .map_err(|e| BotError::InvalidInput(format!("Failed to deserialize job: {}", e)))?;
            Ok(Some(job))
        } else {
            Ok(None)
        }
    }

    pub async fn get_queue_position(&self, discord_id: u64) -> BotResult<Option<usize>> {
        let mut conn = self.database.get_redis_connection().await?;
        let queue: Vec<String> = conn.lrange("job_queue", 0, -1).await?;
        
        for (index, job_json) in queue.iter().enumerate() {
            if let Ok(job) = serde_json::from_str::<QueueJob>(job_json) {
                if job.discord_id == discord_id {
                    return Ok(Some(index + 1));
                }
            }
        }
        
        Ok(None)
    }

    pub async fn get_queue_length(&self) -> BotResult<usize> {
        let mut conn = self.database.get_redis_connection().await?;
        let length: usize = conn.llen("job_queue").await?;
        Ok(length)
    }
}
