use std::error::Error;

use crate::{runtime::validator::is_local_host, telemetry::Telemetry};

pub trait ToolAdapter {
    fn send(&self, endpoint: &str, telemetry: &Telemetry) -> Result<(), Box<dyn Error>>;
}

fn enforce_offline_policy(
    endpoint: &str,
    network_enabled: bool,
    allowed_hosts: &[String],
    telemetry: &Telemetry,
) -> Result<(), Box<dyn Error>> {
    if network_enabled {
        return Ok(());
    }

    let host = endpoint
        .split("//")
        .nth(1)
        .and_then(|v| v.split('/').next())
        .and_then(|v| v.split(':').next())
        .unwrap_or_default();

    if !is_local_host(host, allowed_hosts) {
        telemetry.increment_blocked_network_attempts();
        return Err(format!("blocked external request in offline mode: {endpoint}").into());
    }

    Ok(())
}

pub struct SearchAdapter {
    pub network_enabled: bool,
    pub allowed_hosts: Vec<String>,
}

impl ToolAdapter for SearchAdapter {
    fn send(&self, endpoint: &str, telemetry: &Telemetry) -> Result<(), Box<dyn Error>> {
        enforce_offline_policy(
            endpoint,
            self.network_enabled,
            &self.allowed_hosts,
            telemetry,
        )
    }
}

pub struct FetchAdapter {
    pub network_enabled: bool,
    pub allowed_hosts: Vec<String>,
}

impl ToolAdapter for FetchAdapter {
    fn send(&self, endpoint: &str, telemetry: &Telemetry) -> Result<(), Box<dyn Error>> {
        enforce_offline_policy(
            endpoint,
            self.network_enabled,
            &self.allowed_hosts,
            telemetry,
        )
    }
}

pub struct RagAdapter {
    pub network_enabled: bool,
    pub allowed_hosts: Vec<String>,
}

impl ToolAdapter for RagAdapter {
    fn send(&self, endpoint: &str, telemetry: &Telemetry) -> Result<(), Box<dyn Error>> {
        enforce_offline_policy(
            endpoint,
            self.network_enabled,
            &self.allowed_hosts,
            telemetry,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_external_adapter_requests_and_counts_them() {
        let telemetry = Telemetry::default();
        let adapter = SearchAdapter {
            network_enabled: false,
            allowed_hosts: vec!["localhost".to_string()],
        };

        let err = adapter.send("https://example.org", &telemetry);
        assert!(err.is_err());
        assert_eq!(telemetry.blocked_network_attempts(), 1);
    }
}
