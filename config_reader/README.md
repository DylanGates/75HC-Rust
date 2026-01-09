# Rust Multi-Source Configuration Reader

A comprehensive Rust configuration loader that supports multiple sources with priority ordering:

1. **Command-line arguments** (highest priority)
2. **Environment variables** (APP\_ prefix)
3. **Configuration files** (TOML, JSON, YAML)
4. **Default values** (lowest priority)

## Features

- ✅ **Multiple file formats**: TOML, JSON, YAML
- ✅ **Environment variables**: APP\_ prefixed variables
- ✅ **CLI arguments**: Full clap integration
- ✅ **Priority merging**: CLI > Env > File > Defaults
- ✅ **Validation**: Config validation with descriptive errors
- ✅ **Type safety**: Full Rust type system integration

## Project Structure

```
config_reader/
├── src/main.rs              # Main application with config structs and functions
├── Cargo.toml               # Dependencies (serde, clap, toml, etc.)
├── config.toml              # Sample TOML configuration
├── config.json              # Sample JSON configuration
├── config.yaml              # Sample YAML configuration
├── .env.example             # Environment variables template
└── README.md               # This file
```

## Configuration Structure

The configuration supports these sections:

```rust
AppConfig {
    server: ServerConfig {
        host: String,
        port: u16,
        workers: Option<u32>,
    },
    database: DatabaseConfig {
        host: String,
        port: u16,
        username: String,
        password: String,
        database: String,
        max_connections: Option<u32>,
    },
    logging: LoggingConfig {
        level: String,
        file: Option<String>,
    },
    features: HashMap<String, bool>,
}
```

## Usage Examples

### 1. Load from TOML file

```bash
cargo run -- --config config.toml
```

### 2. Load from JSON file

```bash
cargo run -- --config config.json
```

### 3. Load from YAML file

```bash
cargo run -- --config config.yaml
```

### 4. Override with environment variables

```bash
export APP_SERVER_PORT=9000
export APP_DATABASE_HOST=prod-db.example.com
cargo run -- --config config.toml
```

### 5. Override with CLI arguments

```bash
cargo run -- --config config.toml --server-host 0.0.0.0 --server-port 3000
```

### 6. Full CLI help

```bash
cargo run -- --help
```

## Configuration Sources Priority

1. **CLI Arguments** (highest)

   - `--server-host localhost`
   - `--database-port 3306`

2. **Environment Variables**

   - `APP_SERVER_HOST=localhost`
   - `APP_DATABASE_PORT=3306`

3. **Configuration File**

   - TOML/JSON/YAML file specified by `--config`

4. **Default Values** (lowest)
   - Built-in sensible defaults

## Dependencies

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }    # Serialization
serde_json = "1.0"                                   # JSON support
serde_yaml = "0.9"                                   # YAML support
toml = "0.8"                                         # TOML support
clap = { version = "4.0", features = ["derive"] }    # CLI parsing
anyhow = "1.0"                                       # Error handling
```

## Planned Functions

The implementation will include these key functions:

- `load_config_from_file()` - Load from TOML/JSON/YAML files
- `load_config_from_env()` - Load from APP\_ prefixed environment variables
- `load_config_from_args()` - Load from command-line arguments
- `merge_configs()` - Deep merge configurations with priority
- `validate_config()` - Validate final configuration
- `load_config()` - Main orchestrator function

## File Format Detection

Automatically detects format based on file extension:

- `.toml` → TOML format
- `.json` → JSON format
- `.yaml` or `.yml` → YAML format

## Error Handling

Comprehensive error types:

- `FileNotFound` - Configuration file not found
- `ParseError` - Invalid file format or syntax
- `ValidationError` - Configuration validation failures
- `IoError` - File system errors

## Building and Running

```bash
# Build the project
cargo build

# Run with default config
cargo run

# Run with specific config file
cargo run -- --config config.toml

# Run with CLI overrides
cargo run -- --server-port 3000 --database-host prod-db
```

## Testing

```bash
# Run tests
cargo test

# Run with verbose output
cargo test -- --nocapture
```

## Future Enhancements

- Configuration hot-reloading
- Remote configuration sources (HTTP, etcd, etc.)
- Configuration encryption/decryption
- Configuration schema validation
- Configuration diff and migration tools
