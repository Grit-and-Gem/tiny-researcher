use std::{error::Error, path::Path};

use crate::{config::AppConfig, runtime::validator::is_local_host};

pub fn run_doctor(cfg: &AppConfig) -> Result<(), Box<dyn Error>> {
    if cfg.model_files.is_empty() {
        return Err("doctor failed: no model files configured".into());
    }

    for model_path in &cfg.model_files {
        if !Path::new(model_path).exists() {
            return Err(format!("doctor failed: model file does not exist: {model_path}").into());
        }
    }

    if cfg.remote_api_keys_required {
        return Err("doctor failed: remote API keys are required".into());
    }

    for endpoint in &cfg.endpoints {
        let host = endpoint
            .split("//")
            .nth(1)
            .and_then(|v| v.split('/').next())
            .and_then(|v| v.split(':').next())
            .unwrap_or_default();

        if !is_local_host(host, &cfg.allowed_hosts) {
            return Err(format!("doctor failed: endpoint is non-local: {endpoint}").into());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doctor_rejects_remote_api_keys() {
        let cfg = AppConfig {
            remote_api_keys_required: true,
            model_files: vec!["./Cargo.toml".to_string()],
            ..Default::default()
        };

        assert!(run_doctor(&cfg).is_err());
    }
}
