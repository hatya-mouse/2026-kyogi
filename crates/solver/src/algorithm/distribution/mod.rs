//! Distribute agents using a simple ranking of reachable spots.

mod dijkstra;

use crate::{
    algorithm::{CostMap, PlanningState, Solver, planning_state::SpotState},
    game::{AgentKind, CellId, DayData},
};
use std::collections::{HashMap, HashSet, hash_map::Entry};

struct SpotCandidate {
    pub(super) agent_id: usize,
    pub(super) spot_id: CellId,
    pub(super) steps: u32,
    pub(super) slack: u32,
}

#[derive(Debug, Clone)]
pub(super) struct Assignment {
    pub(super) agent_id: usize,
    pub(super) cell_id: CellId,
}

impl Assignment {
    fn from_candidate(candidate: SpotCandidate) -> Self {
        Self {
            agent_id: candidate.agent_id,
            cell_id: candidate.spot_id,
        }
    }
}

impl Solver<'_> {
    pub(super) fn distribute_agents(
        &self,
        day: &DayData,
        state: &PlanningState,
        cost_map: &CostMap,
        rejected_assignments: &HashSet<(usize, CellId)>,
    ) -> Vec<Assignment> {
        let mut candidates = Vec::new();

        for (agent_id, (agent, cursor)) in day.agents.iter().zip(state.cursor.iter()).enumerate() {
            // Skip non-patrol agents
            if agent.kind != AgentKind::Patrol {
                continue;
            }

            // Calculate the steps to the spots using Dijkstra's algorithm
            let cell_steps = self.dijkstra(cost_map, &cursor.pos);

            for (spot_id, SpotState { stocks, .. }) in &state.spots {
                if rejected_assignments.contains(&(agent_id, *spot_id)) {
                    continue;
                }

                // 1. Do not assign an agent to the spot it is already occupying
                if *spot_id == cursor.pos {
                    continue;
                }

                // 2. If the spot does not have any available stock, skip it
                if *stocks == 0 {
                    continue;
                }

                // 3. Exclude spots that this agent has already visited
                if cursor.visited_spots.contains(spot_id) {
                    continue;
                }

                // Calculate the number of steps it takes to move to the spot
                let steps = cell_steps[spot_id.as_usize()];

                // 4. Exclude unreachable spots
                if steps == u32::MAX {
                    continue;
                }

                // 5. Exclude spots that is unable to reach within a day
                // cursor.fixed_steps + steps = step number when the agent reaches the spot
                if cursor.fixed_steps + steps > day.steps {
                    continue;
                }

                let candidate = SpotCandidate {
                    agent_id,
                    spot_id: *spot_id,
                    steps,
                    slack: day.steps - cursor.fixed_steps - steps,
                };
                candidates.push(candidate);
            }
        }

        // Prioritize unvisited and hard-to-reach brands
        prioritize_candidates(state, &mut candidates);

        // Limit the number of same spots
        limit_candidates_by_stock(state, &mut candidates);

        // Take top candidates and return them
        take_top_candidates(candidates, self.board.agent_count)
    }
}

/// Limits the number of same spots to the number of stocks by removing the distant spots.
fn limit_candidates_by_stock(state: &PlanningState, candidates: &mut Vec<SpotCandidate>) {
    let mut spot_counts: HashMap<CellId, u32> = HashMap::new();
    let mut index = 0;
    while let Some(spot_id) = candidates.get(index).map(|c| c.spot_id) {
        // Increment the spot count
        let spot_count = match spot_counts.entry(spot_id) {
            Entry::Occupied(mut entry) => {
                let count = entry.get_mut();
                *count += 1;
                *count
            }
            Entry::Vacant(entry) => {
                entry.insert(1);
                1
            }
        };

        // If the spot count exceeds the spot's number of stocks, remove the candidate
        let Some(SpotState { stocks, .. }) = state.spots.get(&spot_id) else {
            break;
        };
        if spot_count > *stocks {
            candidates.remove(index);
        } else {
            index += 1;
        }
    }
}

/// Prioritizes scarce unvisited brands and candidates that are close to becoming unreachable.
fn prioritize_candidates(state: &PlanningState, candidates: &mut [SpotCandidate]) {
    const URGENCY_MARGIN: u32 = 5;
    let mut brand_agents = HashMap::new();

    for candidate in candidates.iter() {
        let Some(spot) = state.spots.get(&candidate.spot_id) else {
            continue;
        };
        if !state.unvisited_brands.contains(&spot.brand) {
            continue;
        }

        brand_agents
            .entry(spot.brand)
            .or_insert_with(HashSet::new)
            .insert(candidate.agent_id);
    }

    candidates.sort_unstable_by_key(|candidate| {
        let Some(spot) = state.spots.get(&candidate.spot_id) else {
            return (true, usize::MAX, true, candidate.steps);
        };

        let is_visited = !state.unvisited_brands.contains(&spot.brand);
        let available_agents = brand_agents
            .get(&spot.brand)
            .map_or(usize::MAX, HashSet::len);

        (
            is_visited,
            available_agents,
            candidate.slack > URGENCY_MARGIN,
            candidate.steps,
        )
    });
}

/// Take the top candidates for each agent and convert them to assignments.
fn take_top_candidates(candidates: Vec<SpotCandidate>, agent_count: usize) -> Vec<Assignment> {
    let mut assignments = Vec::new();
    let mut unassigned_agents: HashSet<usize> = (0..agent_count).collect();

    for candidate in candidates {
        if unassigned_agents.remove(&candidate.agent_id) {
            assignments.push(Assignment::from_candidate(candidate));
        }

        if unassigned_agents.is_empty() {
            break;
        }
    }

    assignments
}
