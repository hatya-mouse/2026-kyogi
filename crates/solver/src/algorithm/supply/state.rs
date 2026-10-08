use crate::game::{AgentKind, CellId, DayData};
use std::collections::HashMap;

#[derive(Debug)]
pub(in crate::algorithm) struct SupplyState {
    /// Each supply agent's responsible patrol agents.
    pub supplies: HashMap<usize, SupplyAgentCursor>,
    /// Refill events happend in the day.
    pub refills: Vec<RefillEvent>,
}

impl SupplyState {
    /// Assigns patrol agents to supply agents and creates a new supply state
    pub(in crate::algorithm) fn new(day: &DayData) -> Self {
        Self {
            supplies: assign_patrols(day),
            refills: Vec::new(),
        }
    }
}

#[derive(Debug, Default)]
pub(in crate::algorithm) struct SupplyAgentCursor {
    /// Supply agent's responsible patrol agents.
    pub assigned_patrols: Vec<usize>,
}

#[derive(Debug, Clone, Default)]
pub(in crate::algorithm) struct RefillEvent {
    pub patrol_id: usize,
    pub cell_id: CellId,
    pub step: u32,
}

impl RefillEvent {
    pub(super) fn new(patrol_id: usize, cell_id: CellId, step: u32) -> Self {
        Self {
            patrol_id,
            cell_id,
            step,
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
