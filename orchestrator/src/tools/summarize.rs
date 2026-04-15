use anyhow::Result;

use crate::runtime::llm_client::LlmClient;

#[derive(Clone)]
pub struct Summarizer {
    client: LlmClient,
}

impl Summarizer {
    pub fn new(client: LlmClient) -> Self {
        Self { client }
    }

    pub async fn summarize(&self, topic: &str, context: &str) -> Result<String> {
        let prompt = format!(
            "Summarize research findings about '{topic}'. Use this context:\n{context}\nProvide concise bullets."
        );
        self.client.infer(&prompt).await
    }
}
