use thiserror::Error;

#[derive(Error, Debug)]
pub enum BotError {
    #[error("Database error: {0}")]
    Database(#[from] mongodb::error::Error),
    
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    
    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("Serenity error: {0}")]
    Serenity(#[from] serenity::Error),
    
    #[error("Stripe error: {0}")]
    Stripe(String),
    
    #[error("Pterodactyl API error: {0}")]
    Pterodactyl(String),
    
    #[error("User not found")]
    UserNotFound,
    
    #[error("Insufficient coins: need {needed}, have {available}")]
    InsufficientCoins { needed: i64, available: i64 },
    
    #[error("Server not found")]
    ServerNotFound,
    
    #[error("Coupon not found or expired")]
    CouponInvalid,
    
    #[error("Permission denied")]
    PermissionDenied,
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

pub type BotResult<T> = Result<T, BotError>;
