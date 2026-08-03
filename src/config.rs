use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct DonationConfig {
    pub pix_key: String,
    pub currency: String,
    pub default_values: Vec<u32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub donation: DonationConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            donation: DonationConfig {
                pix_key: "11925416678".to_string(),
                currency: "BRL".to_string(),
                default_values: vec![5, 10, 20, 50, 100],
            },
        }
    }
}

pub fn load_config() -> AppConfig {
    // Try to load from "config.toml" in current working directory first.
    let paths = ["config.toml", "isearch.toml", ".isearch.toml"];
    for path_str in paths.iter() {
        let path = Path::new(path_str);
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(config) = toml::from_str::<AppConfig>(&content) {
                    return config;
                }
            }
        }
    }
    AppConfig::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.donation.pix_key, "11925416678");
        assert_eq!(config.donation.currency, "BRL");
        assert_eq!(config.donation.default_values, vec![5, 10, 20, 50, 100]);
    }

    #[test]
    fn test_load_from_toml() {
        let toml_content = r#"
[donation]
pix_key = "98765432100"
currency = "USD"
default_values = [1, 2, 3]
"#;
        let config: AppConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(config.donation.pix_key, "98765432100");
        assert_eq!(config.donation.currency, "USD");
        assert_eq!(config.donation.default_values, vec![1, 2, 3]);
    }
}
