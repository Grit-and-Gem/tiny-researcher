use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LocalTransport {
    Stdio { command: String, args: Vec<String> },
    Http { endpoint: String },
    FfiStub,
}

#[derive(Debug, Clone)]
pub struct LlmClient {
    transport: LocalTransport,
    http: Client,
}

impl LlmClient {
    pub fn new(transport: LocalTransport) -> Self {
        Self {
            transport,
            http: Client::new(),
        }
    }

    pub async fn infer(&self, prompt: &str) -> Result<String> {
        match &self.transport {
            LocalTransport::Stdio { command, args } => {
                let mut child = Command::new(command)
                    .args(args)
                    .stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::piped())
                    .spawn()?;

                if let Some(stdin) = child.stdin.as_mut() {
                    stdin.write_all(prompt.as_bytes()).await?;
                    stdin.write_all(b"\n").await?;
                }

                let stdout = child
                    .stdout
                    .take()
                    .ok_or_else(|| anyhow!("missing stdout from stdio transport"))?;
                let mut reader = BufReader::new(stdout);
                let mut output = String::new();
                reader.read_line(&mut output).await?;
                Ok(output.trim().to_string())
            }
            LocalTransport::Http { endpoint } => {
                #[derive(Serialize)]
                struct Req<'a> {
                    prompt: &'a str,
                }
                #[derive(Deserialize)]
                struct Resp {
                    output: String,
                }

                let resp = self
                    .http
                    .post(endpoint)
                    .json(&Req { prompt })
                    .send()
                    .await?
                    .error_for_status()?;
                let body: Resp = resp.json().await?;
                Ok(body.output)
            }
            LocalTransport::FfiStub => Ok(format!(
                "[ffi stub] summary unavailable for prompt of {} chars",
                prompt.len()
            )),
        }
    }
}
