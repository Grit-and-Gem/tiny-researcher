use anyhow::{anyhow, Result};

#[derive(Debug, Clone)]
pub struct ScrapeTool {
    pub enabled: bool,
}

impl ScrapeTool {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    pub async fn ingest(&self, uri: &str) -> Result<String> {
        if !self.enabled {
            return Err(anyhow!("scrape disabled by policy"));
        }
        if uri.starts_with("file://") {
            let path = uri.trim_start_matches("file://");
            return Ok(tokio::fs::read_to_string(path).await?);
        }

        Err(anyhow!(
            "only local file:// ingestion is permitted in local-only scrape mode"
        ))
    }
}
