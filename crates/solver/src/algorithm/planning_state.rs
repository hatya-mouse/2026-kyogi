use crate::{
    algorithm::utils::get_action_steps,
    game::{Action, CellId, DayData, DayPlan, Map},
};
use std::num::NonZeroU32;

/// A temporary plan state that is used during planning.
pub(super) struct PlanningState {
    /// A plan that is now being constructed.
    pub plan: DayPlan,
    /// Current temporary state of the agents.
    pub cursor: Vec<AgentCursor>,
}

/// The planned position of the agents, not a server's actual state.
pub(super) struct AgentCursor {
    /// Current position of the agent.
    pub pos: CellId,
    /// Amount of remaining fuels.
    pub fuel: u32,
    /// Number of steps whose plans are already confirmed.
    pub fixed_steps: u32,
}

impl PlanningState {
    pub(super) fn from_day(day: &DayData) -> Self {
        let agent_count = day.agents.len();

        Self {
            plan: DayPlan::new(agent_count),
            cursor: day
                .agents
                .iter()
                .map(|agent| AgentCursor {
                    pos: agent.pos,
                    fuel: agent.fuel,
                    fixed_steps: 0,
                })
                .collect(),
        }
    }

    /// Returns whether the current plan has remaining steps for any agents.
    pub(super) fn has_remaining(&self, day: &DayData) -> bool {
        for cursor in &self.cursor {
            let remaining_steps = day.steps.saturating_sub(cursor.fixed_steps);
            if remaining_steps > 0 {
                return true;
            }
        }
        false
    }

    pub(super) fn try_add_actions(
        &mut self,
        map: &Map,
        day: &DayData,
        agent_id: usize,
        destination: CellId,
        actions: Vec<Action>,
    ) -> bool {
        let Some(cursor) = self.cursor.get_mut(agent_id) else {
            return false;
        };
        let action_steps: u32 = actions
            .iter()
            .map(|action| get_action_steps(map, day, cursor, action))
            .sum();
        if cursor.fixed_steps.saturating_add(action_steps) > day.steps {
            return false;
        }

        self.plan.extend_actions(agent_id, actions);
        cursor.fixed_steps += action_steps;
        cursor.pos = destination;
        true
    }

    /// Pads each agent's unfinished plan with a wait action.
    /// Number of agents and the length of `actions` vector in the plan must be the same.
    /// If not, this function panics.
    ///
    /// # Arguments
    /// - `day`: The day data for the current game day.
    pub(super) fn fill_remaining_waits(&mut self, day: &DayData) {
        assert_eq!(self.plan.actions.len(), self.cursor.len());
        assert_eq!(self.cursor.len(), day.agents.len());

        for (agent_actions, cursor) in self.plan.actions.iter_mut().zip(self.cursor.iter_mut()) {
            let remaining_steps = day.steps.saturating_sub(cursor.fixed_steps);

            // Add wait instruction if the remaining steps is more than 0
            if let Some(wait_steps) = NonZeroU32::new(remaining_steps) {
                agent_actions.push(Action::Wait(wait_steps));
            }

            cursor.fixed_steps = day.steps;
        }
    }
}
