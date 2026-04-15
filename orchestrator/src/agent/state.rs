use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentPhase {
    Idle,
    Planning,
    Gathering,
    Synthesizing,
    Reviewing,
    Done,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    phase: AgentPhase,
    pub step: usize,
    pub max_steps: usize,
    pub budget_remaining: f32,
    pub last_error: Option<String>,
}

impl AgentState {
    pub fn new(max_steps: usize, budget: f32) -> Self {
        Self {
            phase: AgentPhase::Idle,
            step: 0,
            max_steps,
            budget_remaining: budget,
            last_error: None,
        }
    }

    pub fn phase(&self) -> &AgentPhase {
        &self.phase
    }

    pub fn transition(&mut self, next: AgentPhase) -> Result<(), String> {
        use AgentPhase::*;
        let valid = matches!(
            (&self.phase, &next),
            (Idle, Planning)
                | (Planning, Gathering)
                | (Gathering, Synthesizing)
                | (Synthesizing, Reviewing)
                | (Reviewing, Planning)
                | (Reviewing, Done)
                | (_, Failed)
        );

        if valid {
            self.phase = next;
            Ok(())
        } else {
            let err = format!("invalid transition: {:?} -> {:?}", self.phase, next);
            self.last_error = Some(err.clone());
            Err(err)
        }
    }

    pub fn increment_step(&mut self) {
        self.step += 1;
    }

    pub fn spend_budget(&mut self, amount: f32) -> bool {
        if self.budget_remaining >= amount {
            self.budget_remaining -= amount;
            true
        } else {
            false
        }
    }

    pub fn is_exhausted(&self) -> bool {
        self.step >= self.max_steps || self.budget_remaining <= 0.0
    }

    pub fn fail(&mut self, message: impl Into<String>) {
        self.last_error = Some(message.into());
        self.phase = AgentPhase::Failed;
    }
}
