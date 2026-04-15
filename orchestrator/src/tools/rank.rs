use crate::tools::local_search::SearchHit;

#[derive(Debug, Clone, Default)]
pub struct Ranker;

impl Ranker {
    pub fn rerank(&self, topic: &str, mut hits: Vec<SearchHit>) -> Vec<SearchHit> {
        let needle = topic.to_lowercase();
        hits.sort_by(|a, b| {
            let a_boost = if a.snippet.to_lowercase().contains(&needle) {
                1.25
            } else {
                1.0
            };
            let b_boost = if b.snippet.to_lowercase().contains(&needle) {
                1.25
            } else {
                1.0
            };
            (b.score * b_boost)
                .partial_cmp(&(a.score * a_boost))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        hits
    }
}
