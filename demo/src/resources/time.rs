pub use skulpin::app::TimeState;
pub use skulpin::app::TimeContext;

use legion::prelude::*;

#[derive(Copy, Clone)]
pub enum SimulationTimePauseReason {
    Editor = 1,
    User = 2,
}

enum TimeOp {
    SetPaused(bool, SimulationTimePauseReason),
    ResetSimulationTime,
}

// For now just wrap the input helper that skulpin provides
pub struct TimeResource {
    pub time_state: TimeState,
    pub simulation_time: TimeContext,
    pub print_fps_event: skulpin::app::PeriodicEvent,
    pub simulation_pause_flags: u8, // No flags set means simulation is not paused
    pending_time_ops: Vec<TimeOp>,
}

impl TimeResource {
    /// Create a new TimeState. Default is not allowed because the current time affects the object
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        TimeResource {
            time_state: TimeState::new(),
            simulation_time: TimeContext::new(),
            print_fps_event: Default::default(),
            simulation_pause_flags: 0,
            pending_time_ops: Default::default(),
        }
    }

    pub fn system_time(&self) -> &TimeContext {
        self.time_state.app_time_context()
    }

    pub fn game_time(&self) -> &TimeContext {
        &self.simulation_time
    }

    pub fn set_simulation_time_paused(
        &mut self,
        paused: bool,
        reason: SimulationTimePauseReason,
    ) {
        let before = self.is_simulation_paused();
        if paused {
            self.simulation_pause_flags |= (reason as u8);
        } else {
            self.simulation_pause_flags &= !(reason as u8);
        }
        let after = self.is_simulation_paused();
        if before != after {
            log::info!("Simulation pause state change {} -> {}", before, after);
        }
    }

    pub fn reset_simulation_time(&mut self) {
        self.simulation_time = TimeContext::new();
        log::info!("Simulation time reset");
    }

    pub fn is_simulation_paused(&self) -> bool {
        self.simulation_pause_flags != 0
    }

    pub fn advance_time(&mut self) {
        self.time_state.update();
        if !self.is_simulation_paused() {
            self.simulation_time
                .update(self.time_state.previous_update_time());
        }
    }

    pub fn enqueue_set_simulation_time_paused(
        &mut self,
        paused: bool,
        reason: SimulationTimePauseReason,
    ) {
        self.pending_time_ops
            .push(TimeOp::SetPaused(paused, reason));
    }

    pub fn enqueue_reset_simulation_time(&mut self) {
        self.pending_time_ops.push(TimeOp::ResetSimulationTime);
    }

    pub fn process_time_ops(&mut self) {
        let time_ops: Vec<_> = self.pending_time_ops.drain(..).collect();
        for time_op in time_ops {
            match time_op {
                TimeOp::SetPaused(paused, reason) => {
                    self.set_simulation_time_paused(paused, reason)
                }
                TimeOp::ResetSimulationTime => self.reset_simulation_time(),
            }
        }
    }
}
