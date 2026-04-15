mod agent;
mod memory;
mod runtime;
mod tools;

use std::path::PathBuf;

use agent::{AgentConfig, AgentLoop};
use anyhow::Result;
use clap::{Parser, Subcommand};
use runtime::llm_client::{LlmClient, LocalTransport};
use tools::{local_search::LocalSearch, rank::Ranker, scrape::ScrapeTool, summarize::Summarizer};
use tracing::{info, Level};
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Debug, Parser)]
#[command(name = "research", version, about = "Local research orchestrator")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Run the local research loop.
    Run {
        #[arg(long)]
        topic: String,
        #[arg(long, default_value = "gtx1650_4gb")]
        profile: String,
        #[arg(long, default_value_t = 12)]
        max_steps: usize,
        #[arg(long, default_value_t = 24.0)]
        budget: f32,
        #[arg(long, default_value = "./.checkpoints/research.json")]
        checkpoint: PathBuf,
        #[arg(long)]
        resume: bool,
        #[arg(long, default_value = "./corpus")]
        corpus: PathBuf,
        #[arg(long)]
        allow_scrape: bool,
        #[arg(long, default_value = "ffi")]
        llm_backend: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let cli = Cli::parse();
    match cli.command {
        Commands::Run {
            topic,
            profile,
            max_steps,
            budget,
            checkpoint,
            resume,
            corpus,
            allow_scrape,
            llm_backend,
        } => {
            let transport = match llm_backend.as_str() {
                "ffi" => LocalTransport::FfiStub,
                "http" => LocalTransport::Http {
                    endpoint: "http://127.0.0.1:11434/infer".to_string(),
                },
                "stdio" => LocalTransport::Stdio {
                    command: "cat".to_string(),
                    args: vec![],
                },
                other => {
                    info!(backend = %other, "unknown llm backend; defaulting to ffi stub");
                    LocalTransport::FfiStub
                }
            };

            let summarize = Summarizer::new(LlmClient::new(transport));
            let search = LocalSearch::new(corpus);
            let scrape = ScrapeTool::new(allow_scrape);
            let rank = Ranker;

            let mut loop_runner = if resume && checkpoint.exists() {
                info!(path = %checkpoint.display(), "resuming from checkpoint");
                AgentLoop::from_checkpoint(checkpoint, search, scrape, summarize, rank).await?
            } else {
                let cfg = AgentConfig {
                    topic,
                    profile,
                    max_steps,
                    budget,
                    checkpoint_file: checkpoint,
                    allow_scrape,
                };
                AgentLoop::new(cfg, search, scrape, summarize, rank)
            };

            loop_runner.run().await?;
            if let Some(s) = loop_runner.synthesis() {
                println!("{s}");
            }
        }
    }

    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt()
        .with_max_level(Level::INFO)
        .with_env_filter(filter)
        .json()
        .with_current_span(true)
        .with_span_list(true)
        .init();
}
