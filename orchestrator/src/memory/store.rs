use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRecord {
    pub id: String,
    pub text: String,
    pub embedding: Option<Vec<f32>>,
    pub metadata: HashMap<String, String>,
}

pub trait LongTermStore: Send + Sync {
    fn upsert(&mut self, record: MemoryRecord);
    fn search_text(&self, query: &str, limit: usize) -> Vec<MemoryRecord>;
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct InMemoryStore {
    docs: Vec<MemoryRecord>,
}

impl LongTermStore for InMemoryStore {
    fn upsert(&mut self, record: MemoryRecord) {
        if let Some(existing) = self.docs.iter_mut().find(|d| d.id == record.id) {
            *existing = record;
        } else {
            self.docs.push(record);
        }
    }

    fn search_text(&self, query: &str, limit: usize) -> Vec<MemoryRecord> {
        let q = query.to_lowercase();
        self.docs
            .iter()
            .filter(|doc| doc.text.to_lowercase().contains(&q))
            .take(limit)
            .cloned()
            .collect()
    }
}
