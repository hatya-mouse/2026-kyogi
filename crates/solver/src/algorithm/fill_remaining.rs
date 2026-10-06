use std::num::NonZeroU32;

use crate::{
    algorithm::{PlanningState, Solver},
    game::{Action, DayData},
};

impl Solver<'_> {
    /// Pads each agent's unfinished plan with a wait action.
    /// Number of agents and the length of `actions` vector in the plan must be the same.
    /// If not, this function panics.
    ///
    /// # Arguments
    /// - `day`: The day data for the current game day.
    /// - `state`: Planning state after planning other actions.
    pub(super) fn fill_remaining_waits(&self, day: &DayData, state: &mut PlanningState) {
        assert_eq!(state.plan.actions.len(), state.cursor.len());
        assert_eq!(state.cursor.len(), day.agents.len());

        for (agent_actions, cursor) in state.plan.actions.iter_mut().zip(state.cursor.iter_mut()) {
            let remaining_steps = day.steps.saturating_sub(cursor.fixed_steps);

            // Add wait instruction if the remaining steps is more than 0
            if let Some(wait_steps) = NonZeroU32::new(remaining_steps) {
                agent_actions.push(Action::Wait(wait_steps));
            }

            cursor.fixed_steps = day.steps;
        }
    }
}
