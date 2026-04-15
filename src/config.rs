use std::{error::Error, fs};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub network_enabled: bool,
    pub allowed_hosts: Vec<String>,
    pub endpoints: Vec<String>,
    pub model_files: Vec<String>,
    pub remote_api_keys_required: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            network_enabled: true,
            allowed_hosts: vec!["127.0.0.1".to_string(), "localhost".to_string()],
            endpoints: Vec::new(),
            model_files: Vec::new(),
            remote_api_keys_required: false,
        }
    }
}

impl AppConfig {
    pub fn from_toml_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let mut cfg = Self::default();
        let text = fs::read_to_string(path)?;

        for raw in text.lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some(value) = line.strip_prefix("network_enabled") {
                cfg.network_enabled = value.split('=').nth(1).map(str::trim) == Some("true");
            } else if let Some(value) = line.strip_prefix("allowed_hosts") {
                cfg.allowed_hosts = parse_string_array(value.split('=').nth(1).unwrap_or(""));
            } else if let Some(value) = line.strip_prefix("endpoints") {
                cfg.endpoints = parse_string_array(value.split('=').nth(1).unwrap_or(""));
            } else if let Some(value) = line.strip_prefix("model_files") {
                cfg.model_files = parse_string_array(value.split('=').nth(1).unwrap_or(""));
            } else if let Some(value) = line.strip_prefix("remote_api_keys_required") {
                cfg.remote_api_keys_required =
                    value.split('=').nth(1).map(str::trim) == Some("true");
            }
        }

        Ok(cfg)
    }
}

fn parse_string_array(raw: &str) -> Vec<String> {
    raw.trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
        .map(|part| part.trim().trim_matches('"'))
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
        .collect()
}
