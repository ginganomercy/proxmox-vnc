use dotenvy::dotenv;
use std::env;

#[derive(Clone)]
pub struct Config {
    pub port: u16,
    pub proxmox_url: String,
    pub proxmox_token_id: String,
    pub proxmox_token_secret: String,
    pub jwt_secret: String,
}

impl Config {
    pub fn load() -> Self {
        dotenv().ok(); // Ignore if .env doesn't exist

        Self {
            port: env::var("VNC_PORT").unwrap_or_else(|_| "3002".to_string()).parse().unwrap_or(3002),
            proxmox_url: env::var("PROXMOX_URL").expect("PROXMOX_URL must be set"),
            proxmox_token_id: env::var("PROXMOX_TOKEN_ID").expect("PROXMOX_TOKEN_ID must be set"),
            proxmox_token_secret: env::var("PROXMOX_TOKEN_SECRET").expect("PROXMOX_TOKEN_SECRET must be set"),
            jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
        }
    }
}
