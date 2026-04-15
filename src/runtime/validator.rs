use std::error::Error;

use crate::config::AppConfig;

pub fn validate_startup(cfg: &AppConfig) -> Result<(), Box<dyn Error>> {
    if cfg.network_enabled {
        return Ok(());
    }

    for endpoint in &cfg.endpoints {
        let host = endpoint_host(endpoint)?;
        if !is_local_host(&host, &cfg.allowed_hosts) {
            return Err(format!(
                "offline mode forbids non-local endpoint: {endpoint} (host={host})"
            )
            .into());
        }
    }

    Ok(())
}

fn endpoint_host(endpoint: &str) -> Result<String, Box<dyn Error>> {
    if let Some(after_scheme) = endpoint.split("//").nth(1) {
        let host = after_scheme.split('/').next().unwrap_or_default();
        let host_no_port = host.split(':').next().unwrap_or_default();
        if !host_no_port.is_empty() {
            return Ok(host_no_port.to_string());
        }
    }

    Err(format!("invalid endpoint URL: {endpoint}").into())
}

pub fn is_local_host(host: &str, allowed_hosts: &[String]) -> bool {
    let builtins = ["localhost", "127.0.0.1", "::1"];
    builtins.contains(&host) || allowed_hosts.iter().any(|h| h == host)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offline_mode_rejects_non_local_endpoints() {
        let cfg = AppConfig {
            network_enabled: false,
            allowed_hosts: vec!["localhost".to_string()],
            endpoints: vec!["https://example.com/api".to_string()],
            ..Default::default()
        };

        assert!(validate_startup(&cfg).is_err());
    }
}
