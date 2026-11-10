extern crate alloc;

use serde::{Deserialize, Serialize};

/// The distinct phases of Kōgnitara's circadian cognitive cycle,
/// following the 20% Ramp - 60% Deep Core - 20% Crystallization architecture.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircadianPhase {
    /// Front-end wakefulness: direct interaction, perceptions, accumulating experiential deltas
    Awake,
    /// 20% Entry Ramp: sensory gating, unbinding rigid constraints, selecting high-dissonance memories
    DrowsinessRamp,
    /// 60% Core Oniric State: unconstrained lateral exploration, analogy synthesis, paradox resolution
    DeepDream,
    /// 20% Exit Ramp: crystallization, BLAKE3 Merkle integrity tree generation, Kpack & Jinshu commit
    Crystallization,
}

impl CircadianPhase {
    /// Returns the architectural temporal allocation ratio of the sleep/dream gradient (20-60-20).
    pub const fn sleep_gradient_ratio(&self) -> f32 {
        match self {
            CircadianPhase::Awake => 1.0,
            CircadianPhase::DrowsinessRamp => 0.20,
            CircadianPhase::DeepDream => 0.60,
            CircadianPhase::Crystallization => 0.20,
        }
    }
}

/// Manages and tracks the temporal circadian flow and sleep cycles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircadianScheduler {
    /// Current active circadian phase
    pub current_phase: CircadianPhase,
    /// Progress within the current phase (0.0 to 1.0)
    pub phase_progress: f32,
    /// Total cycles completed
    pub cycle_count: u64,
}

impl Default for CircadianScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl CircadianScheduler {
    /// Creates a scheduler starting in the Awake state.
    pub fn new() -> Self {
        Self {
            current_phase: CircadianPhase::Awake,
            phase_progress: 0.0,
            cycle_count: 0,
        }
    }

    /// Advances the cycle clock by an increment.
    /// Returns `Some(new_phase)` when a phase transition occurs.
    pub fn advance(&mut self, step: f32) -> Option<CircadianPhase> {
        self.phase_progress += step;

        if self.phase_progress >= 1.0 {
            self.phase_progress = 0.0;
            let next_phase = match self.current_phase {
                CircadianPhase::Awake => CircadianPhase::DrowsinessRamp,
                CircadianPhase::DrowsinessRamp => CircadianPhase::DeepDream,
                CircadianPhase::DeepDream => CircadianPhase::Crystallization,
                CircadianPhase::Crystallization => {
                    self.cycle_count += 1;
                    CircadianPhase::Awake
                }
            };
            self.current_phase = next_phase.clone();
            Some(next_phase)
        } else {
            None
        }
    }

    /// Manually initiates a transition into sleep/dreaming when cognitive fatigue is elevated.
    pub fn induce_sleep(&mut self) {
        self.current_phase = CircadianPhase::DrowsinessRamp;
        self.phase_progress = 0.0;
    }
}
