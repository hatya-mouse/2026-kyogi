//! Distribute agents using a simple greedy ranking of reachable spots.

mod assignment;
mod dijkstra;

use crate::{
    algorithm::{CostMap, PlanningState, Solver},
    game::{AgentKind, CellId, DayData},
};
use assignment::{AgentCandidates, SpotCandidate};

#[derive(Debug, Clone)]
pub(super) struct Assignment {
    pub(super) agent_id: usize,
    pub(super) spot: CellId,
}

impl Solver<'_> {
    pub(super) fn distribute_agents(
        &self,
        day: &DayData,
        state: &PlanningState,
        cost_map: &CostMap,
    ) -> Vec<Assignment> {
        let mut agents_to_spots = Vec::new();

        for (agent_id, (agent, cursor)) in day.agents.iter().zip(state.cursor.iter()).enumerate() {
            // Skip non-patrol agents
            if agent.kind != AgentKind::Patrol {
                continue;
            }

            // Calculate the steps to the spots using Dijkstra's algorithm
            let cell_steps = self.dijkstra_backward(cost_map, &cursor.pos);

            // Get the distances to spots
            let mut spot_steps = Vec::new();
            for (spot, details) in &self.board.spots {
                // Do not assign an agent to the spot it is already occupying
                if *spot == cursor.pos {
                    continue;
                }

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

                spot_steps.push(SpotCandidate {
                    spot_id: *spot,
                    brand: details.brand().clone(),
                    distance,
                    stock: details.stocks(),
                });
            }

            spot_steps.sort_unstable_by_key(|candidate| (candidate.distance, candidate.spot_id));

            agents_to_spots.push(AgentCandidates {
                agent_id,
                spots: spot_steps,
            });
        }

        // Greedily assign the highest-ranked available candidate.
        self.search(&agents_to_spots)
    }
}
