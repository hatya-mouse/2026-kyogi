use crate::game::{AgentKind, DayData};
use std::collections::HashMap;

#[derive(Debug)]
pub(super) struct SupplyState {
    /// Each supply agent's responsible patrol agents.
    pub supplies: HashMap<usize, SupplyAgentCursor>,
}

impl SupplyState {
    pub(super) fn new(day: &DayData) -> Self {
        Self {
            supplies: assign_patrols(day),
        }
    }

    pub(super) fn reset_current_patrols(&mut self) {
        for supply in self.supplies.values_mut() {
            supply.current_patrol = supply.assigned_patrols.first().copied();
        }
    }
}

#[derive(Debug, Default)]
pub(super) struct SupplyAgentCursor {
    /// Supply agent's responsible patrol agents.
    pub assigned_patrols: Vec<usize>,
    /// Index in `assigned_patrols` of the currently processing patrol agent.
    pub current_patrol: Option<usize>,
}

impl SupplyAgentCursor {
    /// Advances the current patrol to the next assigned patrol.
    pub(super) fn next_patrol(&mut self) {
        if self.assigned_patrols.is_empty() {
            self.current_patrol = None;
        } else if let Some(current_patrol) = self.current_patrol
            && current_patrol + 1 < self.assigned_patrols.len()
        {
            // If the current patrol is that last one, change current_patrol back to 0
            self.current_patrol = Some(current_patrol.saturating_add(1));
        } else {
            self.current_patrol = Some(0);
        }
    }
}

/// Assign patrol agents to supply agents and returns new supply agent state.
fn assign_patrols(day: &DayData) -> HashMap<usize, SupplyAgentCursor> {
    let mut supplies: HashMap<usize, SupplyAgentCursor> = day
        .agents
        .iter()
        .enumerate()
        .filter(|(_, agent)| agent.kind == AgentKind::Supply)
        .map(|(id, _)| (id, SupplyAgentCursor::default()))
        .collect();

    if !supplies.is_empty() {
        let mut patrol_agents: Vec<_> = day
            .agents
            .iter()
            .enumerate()
            .filter(|(_, agent)| agent.kind == AgentKind::Patrol)
            .map(|(id, _)| id)
            .collect();
        let patrols_per_supply = patrol_agents.len() / supplies.len();

        for supply_state in supplies.values_mut() {
            // Take the patrols_per_supply number of patrol agents
            supply_state.assigned_patrols.extend(
                patrol_agents
                    .drain(0..patrols_per_supply)
                    .collect::<Vec<_>>(),
            );
        }

        // Add the remaining patrol agents to the last supply
        if let Some((_, last_supply_state)) = supplies.iter_mut().last() {
            last_supply_state.assigned_patrols.extend(patrol_agents);
        }
    }

    supplies
}
