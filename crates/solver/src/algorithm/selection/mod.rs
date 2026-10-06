mod dijkstra;

use crate::{
    algorithm::{CostMap, PlanningState, Solver},
    game::{AgentKind, CellId, DayData},
};
use std::collections::HashMap;

impl Solver<'_> {
    pub(super) fn distribute_agents(
        &self,
        day: &DayData,
        state: &PlanningState,
        cost_map: &CostMap,
    ) -> Vec<(usize, CellId)> {
        // Store the steps to each agents from each spots
        let spot_to_agents: HashMap<CellId, Vec<(usize, u32)>> = HashMap::new();

        // Calculate the steps to the spots using Dijkstra's algorithm
        for spot_cell in self.spots.keys() {
            let steps = self.dijkstra_backward(cost_map, spot_cell);

            // Collect distances to each agents
            let mut agent_distances = Vec::new();
            for (agent_id, agent) in day.agents.iter().enumerate() {
                // Skip non-patrol agents
                if agent.kind != AgentKind::Patrol {
                    continue;
                }

                // Exclude unreachable agents
                let distance = steps[agent.pos.as_usize()];
                if distance == u32::MAX {
                    continue;
                }

                agent_distances.push((agent_id, distance));
            }

            spot_to_agents.insert(*spot_cell, agent_distances);
        }

        ()
    }
}
