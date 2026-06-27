use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub mongodb: MongoDbConfig,
    pub redis: RedisConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MongoDbConfig {
    pub uri: String,
    pub database: String,
    pub books_collection: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RedisConfig {
    pub url: String,
    pub max_connections: u32,
    pub ttl_seconds: u64,
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let mut cfg = config::Config::builder();

        // Start with environment variables
        cfg = cfg.add_source(config::Environment::default());

        // Add .env file if it exists
        if let Ok(env_file) = env::var("ENV_FILE") {
            cfg = cfg.add_source(config::File::with_name(&env_file));
        } else if std::path::Path::new(".env").exists() {
            cfg = cfg.add_source(config::File::with_name(".env"));
        }

        // Set defaults
        cfg = cfg
            .set_default("server.host", "0.0.0.0")?
            .set_default("server.port", 3000)?
            .set_default("mongodb.database", "book_library")?
            .set_default("mongodb.books_collection", "books")?
            .set_default("redis.max_connections", 10)?
            .set_default("redis.ttl_seconds", 3600)?;

        cfg.build()?.try_deserialize()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 3000,
            },
            mongodb: MongoDbConfig {
                uri: "mongodb://localhost:27017".to_string(),
                database: "book_library".to_string(),
                books_collection: "books".to_string(),
            },
            redis: RedisConfig {
                url: "redis://localhost:6379".to_string(),
                max_connections: 10,
                ttl_seconds: 3600,
            },
        }
    }
}