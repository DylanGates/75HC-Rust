use clap::{Parser, Subcommand};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;

const DATA_FILE: &str = "data.json";

#[derive(Parser)]
#[command(name = "url_shortener")]
#[command(about = "Simple local URL shortener that stores mappings in a file.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a short key for a URL
    Create {
        /// The original URL to shorten
        url: String,
        /// Optional custom alias
        #[arg(short, long)]
        alias: Option<String>,
    },
    /// Resolve a short key to the original URL
    Resolve {
        /// The short key or alias
        key: String,
    },
    /// List all mappings
    List,
    /// Delete a mapping
    Delete {
        key: String,
    },
    /// Export mappings to a file
    Export {
        file: String,
    },
    /// Import mappings from a file (merges)
    Import {
        file: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Mapping {
    original: String,
    hits: u64,
}

#[derive(Serialize, Deserialize, Default)]
struct Store {
    map: HashMap<String, Mapping>,
}

impl Store {
    fn load<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        if !path.as_ref().exists() {
            return Ok(Store::default());
        }
        let mut f = File::open(path)?;
        let mut s = String::new();
        f.read_to_string(&mut s)?;
        let store: Store = serde_json::from_str(&s).unwrap_or_default();
        Ok(store)
    }

    fn save<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let tmp = format!("{}.tmp", path.as_ref().display());
        let mut f = File::create(&tmp)?;
        let s = serde_json::to_string_pretty(self).unwrap();
        f.write_all(s.as_bytes())?;
        fs::rename(tmp, path)?;
        Ok(())
    }

    fn insert(&mut self, key: String, original: String) -> bool {
        if self.map.contains_key(&key) {
            return false;
        }
        self.map.insert(
            key,
            Mapping {
                original,
                hits: 0,
            },
        );
        true
    }

    fn resolve(&mut self, key: &str) -> Option<String> {
        if let Some(m) = self.map.get_mut(key) {
            m.hits = m.hits.saturating_add(1);
            return Some(m.original.clone());
        }
        None
    }
}

fn generate_short_code(len: usize) -> String {
    let mut rng = rand::thread_rng();
    let s: String = (&mut rng)
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect();
    s
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    let mut store = Store::load(DATA_FILE)?;

    match cli.command {
        Commands::Create { url, alias } => {
            let key = if let Some(a) = alias {
                a
            } else {
                // try to generate a unique short code up to a few times
                let mut k;
                let mut tries = 0;
                loop {
                    k = generate_short_code(6);
                    if !store.map.contains_key(&k) {
                        break k;
                    }
                    tries += 1;
                    if tries > 8 {
                        // fallback to longer code
                        k = generate_short_code(8);
                        if !store.map.contains_key(&k) {
                            break k;
                        }
                    }
                }
            };

            if store.insert(key.clone(), url.clone()) {
                store.save(DATA_FILE)?;
                println!("{} -> {}", key, url);
            } else {
                eprintln!("Key '{}' already exists.", key);
                std::process::exit(1);
            }
        }
        Commands::Resolve { key } => {
            if let Some(url) = store.resolve(&key) {
                store.save(DATA_FILE)?;
                println!("{}", url);
            } else {
                eprintln!("Key '{}' not found.", key);
                std::process::exit(2);
            }
        }
        Commands::List => {
            for (k, v) in &store.map {
                println!("{} -> {} (hits={})", k, v.original, v.hits);
            }
        }
        Commands::Delete { key } => {
            if store.map.remove(&key).is_some() {
                store.save(DATA_FILE)?;
                println!("Deleted '{}'.", key);
            } else {
                eprintln!("Key '{}' not found.", key);
                std::process::exit(2);
            }
        }
        Commands::Export { file } => {
            let s = serde_json::to_string_pretty(&store).unwrap();
            fs::write(file, s)?;
            println!("Exported to file.");
        }
        Commands::Import { file } => {
            let content = fs::read_to_string(file)?;
            let other: Store = serde_json::from_str(&content).unwrap_or_default();
            let mut merged = 0usize;
            for (k, v) in other.map {
                if !store.map.contains_key(&k) {
                    store.map.insert(k, v);
                    merged += 1;
                }
            }
            store.save(DATA_FILE)?;
            println!("Imported {} new mappings.", merged);
        }
    }

    Ok(())
}
