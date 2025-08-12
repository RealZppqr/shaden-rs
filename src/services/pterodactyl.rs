use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::config::Config;
use crate::errors::{BotError, BotResult};
use crate::models::{Server, Resources};

#[derive(Debug, Clone)]
pub struct PterodactylClient {
    client: Client,
    base_url: String,
    api_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateServerRequest {
    pub name: String,
    pub user: i64,
    pub egg: i64,
    pub docker_image: String,
    pub startup: String,
    pub environment: serde_json::Value,
    pub limits: ServerLimits,
    pub feature_limits: FeatureLimits,
    pub allocation: AllocationRequest,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerLimits {
    pub memory: i64,
    pub swap: i64,
    pub disk: i64,
    pub io: i64,
    pub cpu: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureLimits {
    pub databases: i64,
    pub allocations: i64,
    pub backups: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AllocationRequest {
    pub default: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PterodactylServer {
    pub id: i64,
    pub external_id: Option<String>,
    pub uuid: String,
    pub identifier: String,
    pub name: String,
    pub description: String,
    pub status: Option<String>,
    pub suspended: bool,
    pub limits: ServerLimits,
    pub feature_limits: FeatureLimits,
}

impl PterodactylClient {
    pub fn new(config: &Config) -> Self {
        Self {
            client: Client::new(),
            base_url: config.pterodactyl_url.clone(),
            api_key: config.pterodactyl_api_key.clone(),
        }
    }

    pub async fn create_server(&self, server: &Server, user_id: i64) -> BotResult<PterodactylServer> {
        let request = CreateServerRequest {
            name: server.name.clone(),
            user: user_id,
            egg: 1, // Default egg ID - should be configurable
            docker_image: "quay.io/pterodactyl/core:java".to_string(),
            startup: "java -Xms128M -Xmx{{SERVER_MEMORY}}M -jar {{SERVER_JARFILE}}".to_string(),
            environment: serde_json::json!({}),
            limits: ServerLimits {
                memory: server.resources.ram,
                swap: 0,
                disk: server.resources.disk,
                io: 500,
                cpu: server.resources.cpu,
            },
            feature_limits: FeatureLimits {
                databases: server.resources.databases,
                allocations: server.resources.allocations,
                backups: server.resources.backups,
            },
            allocation: AllocationRequest {
                default: 1, // Default allocation ID
            },
        };

        let response = self.client
            .post(&format!("{}/api/application/servers", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(BotError::Pterodactyl(format!("Failed to create server: {}", error_text)));
        }

        let server_response: serde_json::Value = response.json().await?;
        let pterodactyl_server: PterodactylServer = serde_json::from_value(
            server_response["attributes"].clone()
        ).map_err(|e| BotError::Pterodactyl(format!("Failed to parse server response: {}", e)))?;

        Ok(pterodactyl_server)
    }

    pub async fn get_server(&self, server_id: i64) -> BotResult<PterodactylServer> {
        let response = self.client
            .get(&format!("{}/api/application/servers/{}", self.base_url, server_id))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Accept", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(BotError::Pterodactyl("Server not found".to_string()));
        }

        let server_response: serde_json::Value = response.json().await?;
        let pterodactyl_server: PterodactylServer = serde_json::from_value(
            server_response["attributes"].clone()
        ).map_err(|e| BotError::Pterodactyl(format!("Failed to parse server response: {}", e)))?;

        Ok(pterodactyl_server)
    }

    pub async fn delete_server(&self, server_id: i64) -> BotResult<()> {
        let response = self.client
            .delete(&format!("{}/api/application/servers/{}", self.base_url, server_id))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Accept", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(BotError::Pterodactyl(format!("Failed to delete server: {}", error_text)));
        }

        Ok(())
    }

    pub async fn power_action(&self, server_id: i64, action: &str) -> BotResult<()> {
        let response = self.client
            .post(&format!("{}/api/application/servers/{}/power", self.base_url, server_id))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(&serde_json::json!({ "signal": action }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(BotError::Pterodactyl(format!("Failed to perform power action: {}", error_text)));
        }

        Ok(())
    }
}
