use std::path::{Path, PathBuf};

use anyhow::Result;
use tokio::fs;

#[derive(Debug, Clone)]
pub struct SearchHit {
    pub source: PathBuf,
    pub snippet: String,
    pub score: f32,
}

#[derive(Debug, Clone)]
pub struct LocalSearch {
    corpus_dir: PathBuf,
}

impl LocalSearch {
    pub fn new(corpus_dir: impl AsRef<Path>) -> Self {
        Self {
            corpus_dir: corpus_dir.as_ref().to_path_buf(),
        }
    }

    pub async fn query(&self, query: &str, limit: usize) -> Result<Vec<SearchHit>> {
        let mut hits = Vec::new();
        let mut entries = fs::read_dir(&self.corpus_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let text = fs::read_to_string(&path).await.unwrap_or_default();
            let lc = text.to_lowercase();
            let needle = query.to_lowercase();
            let count = lc.matches(&needle).count();
            if count > 0 {
                let snippet = text.chars().take(280).collect();
                hits.push(SearchHit {
                    source: path,
                    snippet,
                    score: count as f32,
                });
            }
        }

        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        hits.truncate(limit);
        Ok(hits)
    }
}
