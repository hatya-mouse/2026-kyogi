mod cost_map;
mod distribution;
mod graph;
mod partial;
mod planning_state;
mod route;
mod supply;
mod utils;

use crate::{
    algorithm::{
        cost_map::CostMap,
        graph::AdjGraph,
        planning_state::{AgentCursor, PlanningState},
        utils::get_action_steps,
    },
    game::{Action, Board, CellId, DayData, DayPlan, Map, Spot},
};
use std::collections::{HashMap, HashSet};

// --- SOVLER ---

/// A solver that calculates the plan for a day.
pub struct Solver<'a> {
    /// The board of the game.
    board: Board<'a>,
    /// Adjacent graph for the map.
    adj_graph: AdjGraph,
}

impl<'a> Solver<'a> {
    pub fn new(
        map: &'a Map,
        spots: HashMap<CellId, Spot>,
        agent_count: usize,
        fuel_limit: u32,
    ) -> Self {
        let adj_graph = AdjGraph::build(map);
        Self {
            board: Board::new(map, spots, agent_count, fuel_limit),
            adj_graph,
        }
    }

    /// Create a solve result for the day.
    pub fn solve_day(&self, day: &DayData) -> DayPlan {
        // Create a planning state and cost map
        let mut state = PlanningState::from_day(&self.board, day);
        let cost_map = CostMap::build(self.board.map, day);

        let mut rejected_assignments = HashSet::new();
        loop {
            // Select spots for each agents
            let assignments = self.distribute_agents(day, &state, &cost_map, &rejected_assignments);

            if assignments.is_empty() || !state.has_remaining(day) {
                break;
            }

            let mut added = false;
            for assignment in assignments {
                if !self.try_add_assignment(&mut state, day, &cost_map, &assignment) {
                    rejected_assignments.insert((assignment.agent_id, assignment.spot_id));
                    continue;
                }

                self.visited_spot(&mut state, assignment.agent_id, assignment.spot_id);
                added = true;
                rejected_assignments.clear();
                break;
            }

            // If no actions were added, break the loop
            if !added {
                continue;
            }
        }

        self.add_partial_movements(&mut state, day, &cost_map);
        state.fill_remaining_waits(day);
        state.plan
    }

    /// Calculates the steps required by an action sequence
    fn route_steps(
        &self,
        map: &Map,
        day: &DayData,
        cursor: &AgentCursor,
        actions: &[Action],
    ) -> u32 {
        let mut cursor = cursor.clone();
        actions
            .iter()
            .map(|action| {
                let steps = get_action_steps(map, day, &cursor, action);
                if let Action::Move(direction) = action {
                    let coord = map.get_coord_from_id(cursor.pos);
                    cursor.pos = map.get_id_from_coord(direction.apply_to_coord(coord));
                }
                steps
            })
            .sum()
    }

    fn visited_spot(&self, state: &mut PlanningState, agent_id: usize, spot_id: CellId) {
        // Mark the spot brand as visited today
        if let Some(spot) = self.board.spots.get(&spot_id) {
            state.unvisited_brands.remove(spot.brand());
        }

        // Decrement the stock by 1 from the state
        if let Some(spot_state) = state.spots.get_mut(&spot_id) {
            spot_state.stocks = spot_state.stocks.saturating_sub(1);
        }

        // Add the spot to visited spots
        if let Some(cursor) = state.cursor.get_mut(agent_id) {
            cursor.visited_spots.insert(spot_id);
        }
    }
}
