mod cost_map;
mod distribution;
mod graph;
mod planning_state;
mod route;
mod utils;

use crate::{
    algorithm::{
        cost_map::CostMap, graph::AdjGraph, planning_state::PlanningState, utils::get_action_steps,
    },
    game::{CellId, DayData, DayPlan, Map, Spot},
};
use std::collections::HashMap;

// --- SOVLER ---

/// A solver that calculates the plan for a day.
pub struct Solver<'a> {
    /// The current map of the game.
    map: &'a Map,
    /// Spots on the map.
    spots: HashMap<CellId, Spot>,
    /// Adjacent graph for the map.
    adj_graph: AdjGraph,
}

impl<'a> Solver<'a> {
    pub fn new(map: &'a Map, spots: HashMap<CellId, Spot>) -> Self {
        let adj_graph = AdjGraph::build(map);
        Self {
            map,
            spots,
            adj_graph,
        }
    }

    /// Create a solve result for the day.
    pub fn solve_day(&self, day: &DayData) -> DayPlan {
        // Create a planning state and cost map
        let mut state = PlanningState::from_day(day);
        let cost_map = CostMap::build(self.map, day);

        loop {
            // Select spots for each agents
            let assignments = self.distribute_agents(day, &state, &cost_map);

            if assignments.is_empty() || !state.has_remaining(day) {
                break;
            }

            for assignment in assignments {
                // Create a route to the spot
                if let Some(cursor) = state.cursor.get_mut(assignment.agent_id) {
                    let route = self.get_route(&cost_map, cursor.pos, assignment.spot);
                    let Some(actions) = self.route_to_actions(&route) else {
                        break;
                    };

                    let action_step_count: u32 = actions
                        .iter()
                        .map(|action| get_action_steps(self.map, day, cursor, action))
                        .sum();
                    state.plan.extend_actions(assignment.agent_id, actions);
                    cursor.fixed_steps += action_step_count;
                    cursor.pos = assignment.spot;
                }
            }
        }

        state.fill_remaining_waits(day);
        state.plan
    }
}
