use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Scratchpad {
    notes: Vec<String>,
}

impl Scratchpad {
    pub fn push(&mut self, note: impl Into<String>) {
        self.notes.push(note.into());
    }

    pub fn recent(&self, limit: usize) -> Vec<String> {
        self.notes.iter().rev().take(limit).cloned().collect()
    }

    pub fn all(&self) -> &[String] {
        &self.notes
    }
}
