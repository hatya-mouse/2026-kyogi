//! Distribute the agents to spots by selecting the best combination of agents and spots.

mod assignment;
mod dijkstra;

use crate::{
    algorithm::{CostMap, PlanningState, Solver},
    game::{AgentKind, CellId, DayData},
};

impl Solver<'_> {
    pub(super) fn distribute_agents(
        &self,
        day: &DayData,
        state: &PlanningState,
        cost_map: &CostMap,
    ) -> Vec<(usize, CellId)> {
        // Store the steps to each agents from each spots
        let mut agents_to_spots: Vec<(usize, Vec<(CellId, u32)>)> = Vec::new();

        // Calculate the steps to the spots using Dijkstra's algorithm
        for (agent_id, (agent, cursor)) in day.agents.iter().zip(state.cursor.iter()).enumerate() {
            // Skip non-patrol agents
            if agent.kind != AgentKind::Patrol {
                continue;
            }

            let cell_steps = self.dijkstra_backward(cost_map, &cursor.pos);

            // Get the distances to spots
            let mut spot_steps = Vec::new();
            for spot in self.spots.keys() {
                let distance = cell_steps[spot.as_usize()];

                // Exclude unreachable spots
                if distance == u32::MAX {
                    continue;
                }

                // Exclude spots that is unable to reach within a day
                // cursor.fixed_steps + distance = step number when the agent reaches the spot
                if cursor.fixed_steps + distance > day.steps {
                    continue;
                }

                spot_steps.push((*spot, distance));
            }

            // Sort the spots by distance
            spot_steps.sort_by_key(|a| a.1);

            agents_to_spots.push((agent_id, spot_steps));
        }

        // Search the best combination of agents and spots
        self.search(&agents_to_spots)
    }
}
