use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
    pub log_level: String,
    pub cache_max_capacity: u64,
    pub auth_username: String,
    pub auth_password: String,
}

impl Config {
    pub fn from_env() -> Self {
        Config {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://localhost/agamenon".to_string()),
            server_host: env::var("SERVER_HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8081".to_string())
                .parse()
                .unwrap_or(8081),
            log_level: env::var("LOG_LEVEL")
                .unwrap_or_else(|_| "info".to_string()),
            cache_max_capacity: env::var("CACHE_MAX_CAPACITY")
                .unwrap_or_else(|_| "1000000".to_string())
                .parse()
                .unwrap_or(1_000_000),
            auth_username: env::var("SCHEMA_REGISTRY_USER")
                .unwrap_or_else(|_| "admin".to_string()),
            auth_password: env::var("SCHEMA_REGISTRY_PASSWORD")
                .unwrap_or_else(|_| "admin".to_string()),
        }
    }
}
