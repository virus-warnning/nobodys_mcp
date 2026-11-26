use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server_name: String,
    pub version: String,
    pub host: String,
    pub port: u16,
    pub endpoint: String,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let data = std::fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&data)?;
        Ok(config)
    }

    pub fn load_or_default() -> Result<Self, Box<dyn std::error::Error>> {
        let path = Path::new("config.json");
        if path.exists() {
            Self::load(path)
        } else {
            Ok(Self::default())
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_name: "nobodys_mcp".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            host: "0.0.0.0".to_string(),
            port: 55851,
            endpoint: "/mcp".to_string(),
        }
    }
}
