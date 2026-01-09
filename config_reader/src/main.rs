use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use clap::{Arg, Command};
use toml;
use serde_json;
use serde_yaml;

/// Configuration structure that can be loaded from multiple sources
/// Supports TOML, JSON, YAML files, environment variables, and CLI arguments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
    pub features: HashMap<String, bool>,
}

/// Server configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: Option<u32>,
}

/// Database configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub max_connections: Option<u32>,
}

/// Logging configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file: Option<String>,
}

/// Supported configuration file formats
#[derive(Debug, Clone)]
pub enum ConfigFormat {
    Toml,
    Json,
    Yaml,
}

/// Error type for configuration operations
#[derive(Debug)]
pub enum ConfigError {
    FileNotFound(String),
    ParseError(String),
    ValidationError(String),
    IoError(std::io::Error),
}

/// Load configuration from a file (TOML, JSON, or YAML)
/// Automatically detects format based on file extension
/// Returns the parsed configuration or an error
fn load_config_from_file<P: AsRef<Path>>(file_path: P) -> Result<AppConfig, ConfigError> {
    let file_path = file_path.as_ref();

    if !file_path.exists() {
        return Err(ConfigError::FileNotFound(file_path.to_string_lossy().to_string()));
    }

    let format = detect_format_from_extension(file_path)
        .ok_or_else(|| ConfigError::ParseError("Unsupported file format".to_string()))?;

    let contents = fs::read_to_string(file_path)
        .map_err(ConfigError::IoError)?;

    match format {
        ConfigFormat::Toml => {
            toml::from_str(&contents)
                .map_err(|e| ConfigError::ParseError(format!("TOML parse error: {}", e)))
        }
        ConfigFormat::Json => {
            serde_json::from_str(&contents)
                .map_err(|e| ConfigError::ParseError(format!("JSON parse error: {}", e)))
        }
        ConfigFormat::Yaml => {
            serde_yaml::from_str(&contents)
                .map_err(|e| ConfigError::ParseError(format!("YAML parse error: {}", e)))
        }
    }
}

/// Load configuration from environment variables
/// Looks for variables with APP_ prefix (e.g., APP_SERVER_HOST, APP_DATABASE_PORT)
/// Merges with existing config if provided
fn load_config_from_env(existing_config: Option<AppConfig>) -> Result<AppConfig, ConfigError> {
    let mut config = existing_config.unwrap_or_else(create_default_config);

    // Server configuration
    if let Ok(host) = env::var("APP_SERVER_HOST") {
        config.server.host = host;
    }
    if let Ok(port_str) = env::var("APP_SERVER_PORT") {
        config.server.port = port_str.parse()
            .map_err(|_| ConfigError::ParseError("Invalid APP_SERVER_PORT".to_string()))?;
    }
    if let Ok(workers_str) = env::var("APP_SERVER_WORKERS") {
        config.server.workers = Some(workers_str.parse()
            .map_err(|_| ConfigError::ParseError("Invalid APP_SERVER_WORKERS".to_string()))?);
    }

    // Database configuration
    if let Ok(host) = env::var("APP_DATABASE_HOST") {
        config.database.host = host;
    }
    if let Ok(port_str) = env::var("APP_DATABASE_PORT") {
        config.database.port = port_str.parse()
            .map_err(|_| ConfigError::ParseError("Invalid APP_DATABASE_PORT".to_string()))?;
    }
    if let Ok(username) = env::var("APP_DATABASE_USERNAME") {
        config.database.username = username;
    }
    if let Ok(password) = env::var("APP_DATABASE_PASSWORD") {
        config.database.password = password;
    }
    if let Ok(database) = env::var("APP_DATABASE_DATABASE") {
        config.database.database = database;
    }
    if let Ok(max_conn_str) = env::var("APP_DATABASE_MAX_CONNECTIONS") {
        config.database.max_connections = Some(max_conn_str.parse()
            .map_err(|_| ConfigError::ParseError("Invalid APP_DATABASE_MAX_CONNECTIONS".to_string()))?);
    }

    // Logging configuration
    if let Ok(level) = env::var("APP_LOGGING_LEVEL") {
        config.logging.level = level;
    }
    if let Ok(file) = env::var("APP_LOGGING_FILE") {
        config.logging.file = Some(file);
    }

    // Feature flags
    for (key, _) in env::vars() {
        if key.starts_with("APP_FEATURES_") {
            let feature_name = key.strip_prefix("APP_FEATURES_").unwrap().to_lowercase();
            if let Ok(value_str) = env::var(&key) {
                if let Ok(value) = value_str.parse::<bool>() {
                    config.features.insert(feature_name, value);
                }
            }
        }
    }

    Ok(config)
}

/// Load configuration from command line arguments
/// Uses clap to define and parse CLI arguments
/// Highest priority - overrides file and env configs
fn load_config_from_args() -> Result<AppConfig, ConfigError> {
    let matches = Command::new("Config Reader")
        .version("1.0")
        .author("Rust Config Reader")
        .about("Multi-source configuration loader")
        .arg(
            Arg::new("config")
                .long("config")
                .short('c')
                .value_name("FILE")
                .help("Configuration file (TOML, JSON, or YAML)")
        )
        .arg(
            Arg::new("server-host")
                .long("server-host")
                .value_name("HOST")
                .help("Server host address")
        )
        .arg(
            Arg::new("server-port")
                .long("server-port")
                .value_name("PORT")
                .help("Server port number")
        )
        .arg(
            Arg::new("server-workers")
                .long("server-workers")
                .value_name("WORKERS")
                .help("Number of server workers")
        )
        .arg(
            Arg::new("database-host")
                .long("database-host")
                .value_name("HOST")
                .help("Database host address")
        )
        .arg(
            Arg::new("database-port")
                .long("database-port")
                .value_name("PORT")
                .help("Database port number")
        )
        .arg(
            Arg::new("database-username")
                .long("database-username")
                .value_name("USERNAME")
                .help("Database username")
        )
        .arg(
            Arg::new("database-password")
                .long("database-password")
                .value_name("PASSWORD")
                .help("Database password")
        )
        .arg(
            Arg::new("database-name")
                .long("database-name")
                .value_name("NAME")
                .help("Database name")
        )
        .arg(
            Arg::new("database-max-connections")
                .long("database-max-connections")
                .value_name("MAX")
                .help("Maximum database connections")
        )
        .arg(
            Arg::new("logging-level")
                .long("logging-level")
                .value_name("LEVEL")
                .help("Logging level (debug, info, warn, error)")
        )
        .arg(
            Arg::new("logging-file")
                .long("logging-file")
                .value_name("FILE")
                .help("Log file path")
        )
        .get_matches();

    let mut config = create_default_config();

    // Server configuration
    if let Some(host) = matches.get_one::<String>("server-host") {
        config.server.host = host.clone();
    }
    if let Some(port_str) = matches.get_one::<String>("server-port") {
        config.server.port = port_str.parse()
            .map_err(|_| ConfigError::ParseError("Invalid server port".to_string()))?;
    }
    if let Some(workers_str) = matches.get_one::<String>("server-workers") {
        config.server.workers = Some(workers_str.parse()
            .map_err(|_| ConfigError::ParseError("Invalid server workers".to_string()))?);
    }

    // Database configuration
    if let Some(host) = matches.get_one::<String>("database-host") {
        config.database.host = host.clone();
    }
    if let Some(port_str) = matches.get_one::<String>("database-port") {
        config.database.port = port_str.parse()
            .map_err(|_| ConfigError::ParseError("Invalid database port".to_string()))?;
    }
    if let Some(username) = matches.get_one::<String>("database-username") {
        config.database.username = username.clone();
    }
    if let Some(password) = matches.get_one::<String>("database-password") {
        config.database.password = password.clone();
    }
    if let Some(database) = matches.get_one::<String>("database-name") {
        config.database.database = database.clone();
    }
    if let Some(max_conn_str) = matches.get_one::<String>("database-max-connections") {
        config.database.max_connections = Some(max_conn_str.parse()
            .map_err(|_| ConfigError::ParseError("Invalid max connections".to_string()))?);
    }

    // Logging configuration
    if let Some(level) = matches.get_one::<String>("logging-level") {
        config.logging.level = level.clone();
    }
    if let Some(file) = matches.get_one::<String>("logging-file") {
        config.logging.file = Some(file.clone());
    }

    Ok(config)
}

/// Merge multiple configuration sources with priority order
/// Priority: CLI args > Environment variables > Config file
/// Later sources override earlier ones for conflicting keys
fn merge_configs(base: AppConfig, overrides: AppConfig) -> AppConfig {
    let mut merged = base;

    // Merge server config
    merged.server.host = overrides.server.host;
    merged.server.port = overrides.server.port;
    if overrides.server.workers.is_some() {
        merged.server.workers = overrides.server.workers;
    }

    // Merge database config
    merged.database.host = overrides.database.host;
    merged.database.port = overrides.database.port;
    merged.database.username = overrides.database.username;
    merged.database.password = overrides.database.password;
    merged.database.database = overrides.database.database;
    if overrides.database.max_connections.is_some() {
        merged.database.max_connections = overrides.database.max_connections;
    }

    // Merge logging config
    merged.logging.level = overrides.logging.level;
    if overrides.logging.file.is_some() {
        merged.logging.file = overrides.logging.file;
    }

    // Merge features (overrides take precedence)
    for (key, value) in overrides.features {
        merged.features.insert(key, value);
    }

    merged
}

/// Validate the final configuration
/// Checks for required fields, valid ranges, and logical consistency
fn validate_config(config: &AppConfig) -> Result<(), ConfigError> {
    // Validate server configuration
    if config.server.host.is_empty() {
        return Err(ConfigError::ValidationError("Server host cannot be empty".to_string()));
    }
    if config.server.port == 0 {
        return Err(ConfigError::ValidationError("Server port must be greater than 0".to_string()));
    }
    if let Some(workers) = config.server.workers {
        if workers == 0 {
            return Err(ConfigError::ValidationError("Server workers must be greater than 0".to_string()));
        }
    }

    // Validate database configuration
    if config.database.host.is_empty() {
        return Err(ConfigError::ValidationError("Database host cannot be empty".to_string()));
    }
    if config.database.port == 0 {
        return Err(ConfigError::ValidationError("Database port must be greater than 0".to_string()));
    }
    if config.database.username.is_empty() {
        return Err(ConfigError::ValidationError("Database username cannot be empty".to_string()));
    }
    if config.database.database.is_empty() {
        return Err(ConfigError::ValidationError("Database name cannot be empty".to_string()));
    }
    if let Some(max_conn) = config.database.max_connections {
        if max_conn == 0 {
            return Err(ConfigError::ValidationError("Database max connections must be greater than 0".to_string()));
        }
    }

    // Validate logging configuration
    let valid_levels = ["debug", "info", "warn", "error"];
    if !valid_levels.contains(&config.logging.level.as_str()) {
        return Err(ConfigError::ValidationError(format!("Invalid logging level: {}", config.logging.level)));
    }

    // Check for logical inconsistencies
    if config.database.password.is_empty() && config.database.host != "localhost" {
        eprintln!("Warning: Empty database password used with non-localhost host");
    }

    Ok(())
}

/// Determine config file format from file extension
fn detect_format_from_extension(file_path: &Path) -> Option<ConfigFormat> {
    file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .and_then(|ext_str| match ext_str.to_lowercase().as_str() {
            "toml" => Some(ConfigFormat::Toml),
            "json" => Some(ConfigFormat::Json),
            "yaml" | "yml" => Some(ConfigFormat::Yaml),
            _ => None,
        })
}

/// Create default configuration with sensible defaults
fn create_default_config() -> AppConfig {
    let mut features = HashMap::new();
    features.insert("debug_mode".to_string(), false);
    features.insert("metrics".to_string(), true);
    features.insert("cache".to_string(), true);

    AppConfig {
        server: ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
            workers: Some(4),
        },
        database: DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            username: "postgres".to_string(),
            password: "".to_string(),
            database: "myapp".to_string(),
            max_connections: Some(10),
        },
        logging: LoggingConfig {
            level: "info".to_string(),
            file: Some("app.log".to_string()),
        },
        features,
    }
}

/// Main configuration loading function
/// Orchestrates loading from all sources in priority order
/// Priority: CLI args > Environment > Config file > Defaults
fn load_config() -> Result<AppConfig, ConfigError> {
    // TODO: Load defaults first
    // TODO: Try to load from config file if specified
    // TODO: Load from environment variables
    // TODO: Load from CLI arguments (highest priority)
    // TODO: Merge all sources
    // TODO: Validate final configuration
    // TODO: Return final merged and validated config
}

/// Display configuration in a human-readable format
/// Useful for debugging and verification
fn print_config(config: &AppConfig) {
    // TODO: Pretty-print the configuration
    // TODO: Show all sections and values
    // TODO: Handle sensitive data (passwords) appropriately
}

/// Main application entry point
/// Demonstrates configuration loading and usage
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Load configuration using load_config()
    // TODO: Handle configuration errors gracefully
    // TODO: Print loaded configuration
    // TODO: Demonstrate configuration usage
    // TODO: Exit with appropriate status code
}
