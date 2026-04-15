use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::{info, instrument, warn};

use crate::{
    agent::state::{AgentPhase, AgentState},
    memory::{scratchpad::Scratchpad, store::InMemoryStore},
    tools::{local_search::LocalSearch, rank::Ranker, scrape::ScrapeTool, summarize::Summarizer},
};

#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub topic: String,
    pub profile: String,
    pub max_steps: usize,
    pub budget: f32,
    pub checkpoint_file: PathBuf,
    pub allow_scrape: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub state: AgentState,
    pub scratchpad: Scratchpad,
    pub synthesis: Option<String>,
    pub topic: String,
    pub profile: String,
}

pub struct AgentLoop {
    config: AgentConfig,
    state: AgentState,
    search: LocalSearch,
    scrape: ScrapeTool,
    summarize: Summarizer,
    rank: Ranker,
    scratchpad: Scratchpad,
    _store: InMemoryStore,
    synthesis: Option<String>,
}

impl AgentLoop {
    pub fn new(
        config: AgentConfig,
        search: LocalSearch,
        scrape: ScrapeTool,
        summarize: Summarizer,
        rank: Ranker,
    ) -> Self {
        Self {
            state: AgentState::new(config.max_steps, config.budget),
            _store: InMemoryStore::default(),
            scratchpad: Scratchpad::default(),
            synthesis: None,
            config,
            search,
            scrape,
            summarize,
            rank,
        }
    }

    pub async fn from_checkpoint(
        path: impl AsRef<Path>,
        search: LocalSearch,
        scrape: ScrapeTool,
        summarize: Summarizer,
        rank: Ranker,
    ) -> Result<Self> {
        let checkpoint_file = path.as_ref().to_path_buf();
        let bytes = fs::read(&checkpoint_file).await?;
        let cp: Checkpoint = serde_json::from_slice(&bytes)?;
        let config = AgentConfig {
            topic: cp.topic.clone(),
            profile: cp.profile.clone(),
            max_steps: cp.state.max_steps,
            budget: cp.state.budget_remaining,
            checkpoint_file,
            allow_scrape: scrape.enabled,
        };

        Ok(Self {
            config,
            state: cp.state,
            search,
            scrape,
            summarize,
            rank,
            scratchpad: cp.scratchpad,
            _store: InMemoryStore::default(),
            synthesis: cp.synthesis,
        })
    }

    #[instrument(skip(self))]
    pub async fn run(&mut self) -> Result<()> {
        if matches!(self.state.phase(), AgentPhase::Idle) {
            self.state.transition(AgentPhase::Planning)?;
        }

        while !self.state.is_exhausted() {
            match self.state.phase() {
                AgentPhase::Planning => self.step_planning().await?,
                AgentPhase::Gathering => self.step_gathering().await?,
                AgentPhase::Synthesizing => self.step_synthesizing().await?,
                AgentPhase::Reviewing => {
                    if self.step_reviewing().await? {
                        self.state.transition(AgentPhase::Done)?;
                        break;
                    }
                }
                AgentPhase::Done | AgentPhase::Failed => break,
                AgentPhase::Idle => self.state.transition(AgentPhase::Planning)?,
            }

            self.state.increment_step();
            self.persist_checkpoint().await?;
        }

        if self.state.is_exhausted() && !matches!(self.state.phase(), AgentPhase::Done) {
            warn!("agent stopped due to max-step or budget exhaustion");
            self.state.fail("resource exhaustion");
            self.persist_checkpoint().await?;
        }

        Ok(())
    }

    async fn step_planning(&mut self) -> Result<()> {
        info!(topic = %self.config.topic, profile = %self.config.profile, "planning step");
        if !self.state.spend_budget(1.0) {
            return Ok(());
        }
        self.scratchpad.push(format!(
            "Plan step {} for topic {}",
            self.state.step, self.config.topic
        ));
        self.state.transition(AgentPhase::Gathering)?;
        Ok(())
    }

    async fn step_gathering(&mut self) -> Result<()> {
        info!("gathering step");
        if !self.state.spend_budget(2.0) {
            return Ok(());
        }

        let hits = self.search.query(&self.config.topic, 8).await?;
        let hits = self.rank.rerank(&self.config.topic, hits);
        for hit in hits.into_iter().take(3) {
            self.scratchpad.push(format!(
                "Source: {}\nScore: {:.2}\n{}",
                hit.source.display(),
                hit.score,
                hit.snippet
            ));
        }

        if self.config.allow_scrape {
            let sample_uri = format!("file://{}/README.md", std::env::current_dir()?.display());
            if let Ok(content) = self.scrape.ingest(&sample_uri).await {
                self.scratchpad.push(format!(
                    "Optional local ingest from {}: {}",
                    sample_uri,
                    content.chars().take(160).collect::<String>()
                ));
            }
        }

        self.state.transition(AgentPhase::Synthesizing)?;
        Ok(())
    }

    async fn step_synthesizing(&mut self) -> Result<()> {
        info!("synthesizing step");
        if !self.state.spend_budget(3.0) {
            return Ok(());
        }

        let context = self.scratchpad.recent(8).join("\n---\n");
        let summary = self
            .summarize
            .summarize(&self.config.topic, &context)
            .await?;
        self.synthesis = Some(summary.clone());
        self.scratchpad.push(format!("Draft synthesis:\n{summary}"));

        self.state.transition(AgentPhase::Reviewing)?;
        Ok(())
    }

    async fn step_reviewing(&mut self) -> Result<bool> {
        info!("reviewing step");
        if !self.state.spend_budget(1.0) {
            return Ok(false);
        }

        let quality_gate = self
            .synthesis
            .as_ref()
            .map(|s| s.len() > 40)
            .unwrap_or(false);

        if quality_gate {
            self.scratchpad
                .push("Review accepted synthesis".to_string());
            Ok(true)
        } else {
            self.scratchpad
                .push("Review requested another iteration".to_string());
            self.state.transition(AgentPhase::Planning)?;
            Ok(false)
        }
    }

    pub async fn persist_checkpoint(&self) -> Result<()> {
        let cp = Checkpoint {
            state: self.state.clone(),
            scratchpad: self.scratchpad.clone(),
            synthesis: self.synthesis.clone(),
            topic: self.config.topic.clone(),
            profile: self.config.profile.clone(),
        };

        if let Some(parent) = self.config.checkpoint_file.parent() {
            fs::create_dir_all(parent).await?;
        }

        let bytes = serde_json::to_vec_pretty(&cp)?;
        fs::write(&self.config.checkpoint_file, bytes).await?;
        Ok(())
    }

    pub fn synthesis(&self) -> Option<&str> {
        self.synthesis.as_deref()
    }
}
