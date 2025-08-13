use anyhow::Result;
use mongodb::{Client, Database as MongoDatabase, Collection};
use redis::{Client as RedisClient, aio::Connection as RedisConnection};
use crate::config::Config;
use crate::models::*;
use crate::errors::{BotError, BotResult};
use mongodb::bson::{doc, Document};
use futures::TryStreamExt;

#[derive(Clone)]
pub struct Database {
    mongo: MongoDatabase,
    redis_client: RedisClient,
}

impl Database {
    pub async fn new(config: &Config) -> Result<Self> {
        let mongo_client = Client::with_uri_str(&config.mongodb_uri).await?;
        let mongo = mongo_client.database("shaden_rs");
        
        let redis_client = RedisClient::open(config.redis_uri.as_str())?;

        Ok(Self {
            mongo,
            redis_client,
        })
    }

    pub async fn get_redis_connection(&self) -> BotResult<RedisConnection> {
        Ok(self.redis_client.get_async_connection().await?)
    }

    // User operations
    pub fn users(&self) -> Collection<User> {
        self.mongo.collection("users")
    }

    pub async fn get_user(&self, discord_id: u64) -> BotResult<Option<User>> {
        let user = self.users()
            .find_one(doc! { "discord_id": discord_id as i64 }, None)
            .await?;
        Ok(user)
    }

    pub async fn create_user(&self, discord_id: u64) -> BotResult<User> {
        let user = User::new(discord_id);
        self.users().insert_one(&user, None).await?;
        Ok(user)
    }

    pub async fn update_user(&self, user: &User) -> BotResult<()> {
        self.users()
            .replace_one(doc! { "discord_id": user.discord_id as i64 }, user, None)
            .await?;
        Ok(())
    }

    pub async fn get_or_create_user(&self, discord_id: u64) -> BotResult<User> {
        if let Some(user) = self.get_user(discord_id).await? {
            Ok(user)
        } else {
            self.create_user(discord_id).await
        }
    }

    // Server operations
    pub fn servers(&self) -> Collection<Server> {
        self.mongo.collection("servers")
    }

    pub async fn create_server(&self, server: &Server) -> BotResult<()> {
        self.servers().insert_one(server, None).await?;
        Ok(())
    }

    pub async fn get_user_servers(&self, discord_id: u64) -> BotResult<Vec<Server>> {
        let cursor = self.servers()
            .find(doc! { "discord_id": discord_id as i64 }, None)
            .await?;
        let servers: Vec<Server> = cursor.try_collect().await?;
        Ok(servers)
    }

    pub async fn get_server(&self, server_id: &str) -> BotResult<Option<Server>> {
        let uuid = uuid::Uuid::parse_str(server_id)
            .map_err(|_| BotError::InvalidInput("Invalid server ID".to_string()))?;
        
        let server = self.servers()
            .find_one(doc! { "id": uuid.to_string() }, None)
            .await?;
        Ok(server)
    }

    pub async fn update_server(&self, server: &Server) -> BotResult<()> {
        self.servers()
            .replace_one(doc! { "id": server.id.to_string() }, server, None)
            .await?;
        Ok(())
    }

    pub async fn delete_server(&self, server_id: &str) -> BotResult<()> {
        let uuid = uuid::Uuid::parse_str(server_id)
            .map_err(|_| BotError::InvalidInput("Invalid server ID".to_string()))?;
        
        self.servers()
            .delete_one(doc! { "id": uuid.to_string() }, None)
            .await?;
        Ok(())
    }

    // Coupon operations
    pub fn coupons(&self) -> Collection<Coupon> {
        self.mongo.collection("coupons")
    }

    pub async fn get_coupon(&self, code: &str) -> BotResult<Option<Coupon>> {
        let coupon = self.coupons()
            .find_one(doc! { "code": code }, None)
            .await?;
        Ok(coupon)
    }

    pub async fn create_coupon(&self, coupon: &Coupon) -> BotResult<()> {
        self.coupons().insert_one(coupon, None).await?;
        Ok(())
    }

    pub async fn update_coupon(&self, coupon: &Coupon) -> BotResult<()> {
        self.coupons()
            .replace_one(doc! { "code": &coupon.code }, coupon, None)
            .await?;
        Ok(())
    }

    pub async fn delete_coupon(&self, code: &str) -> BotResult<()> {
        self.coupons()
            .delete_one(doc! { "code": code }, None)
            .await?;
        Ok(())
    }

    // Order operations
    pub fn orders(&self) -> Collection<Order> {
        self.mongo.collection("orders")
    }

    pub async fn create_order(&self, order: &Order) -> BotResult<()> {
        self.orders().insert_one(order, None).await?;
        Ok(())
    }

    pub async fn get_order_by_session(&self, session_id: &str) -> BotResult<Option<Order>> {
        let order = self.orders()
            .find_one(doc! { "stripe_session_id": session_id }, None)
            .await?;
        Ok(order)
    }

    pub async fn update_order(&self, order: &Order) -> BotResult<()> {
        self.orders()
            .replace_one(doc! { "id": order.id.to_string() }, order, None)
            .await?;
        Ok(())
    }

    pub async fn renew_server(&self, server_id: &str, duration_days: u32, cost: i64) -> BotResult<()> {
        let uuid = uuid::Uuid::parse_str(server_id)
            .map_err(|_| BotError::InvalidInput("Invalid server ID".to_string()))?;
        
        let mut server = self.get_server(server_id).await?
            .ok_or(BotError::ServerNotFound)?;
        
        // Extend the server expiration
        server.expires_at = server.expires_at + chrono::Duration::days(duration_days as i64);
        
        self.update_server(&server).await?;
        Ok(())
    }
}
