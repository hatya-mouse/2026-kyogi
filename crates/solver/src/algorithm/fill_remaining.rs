use std::num::NonZeroU32;

use crate::{
    algorithm::{PlanningState, Solver},
    game::{Action, DayData},
};

impl Solver<'_> {
    /// Fills the remaining steps with wait instruction.
    ///
    /// # Arguments
    /// - `steps`: Number of steps required for the day.
    pub(super) fn fill_remaining_waits(&self, day: &DayData, state: &mut PlanningState) {
        for (agent_actions, cursor) in state.plan.actions.iter_mut().zip(state.cursor.iter()) {
            let all_steps: u32 = agent_actions
                .iter()
                .map(|action| self.get_action_steps(day, cursor, action))
                .sum();
            let remaining_steps = day.steps.saturating_sub(all_steps);

            // Add wait instruction if the remaining steps is more than 1
            if let Some(wait_steps) = NonZeroU32::new(remaining_steps) {
                agent_actions.push(Action::Wait(wait_steps));
            }
        }
    }
}
