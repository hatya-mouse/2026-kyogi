mod cost_map;
mod distribution;
mod fill_remaining;
mod graph;
mod route;
mod utils;

use crate::{
    algorithm::{cost_map::CostMap, graph::AdjGraph},
    game::{CellId, DayData, DayPlan, Map, Spot},
};
use std::collections::HashMap;

/// A temporary plan state that is used during planning.
struct PlanningState {
    /// A plan that is now being constructed.
    plan: DayPlan,
    /// Current temporary state of the agents.
    cursor: Vec<AgentCursor>,
}

impl PlanningState {
    fn from_day(day: &DayData) -> Self {
        let agent_count = day.agents.len();

        Self {
            plan: DayPlan {
                actions: vec![Vec::new(); agent_count],
            },
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
}

/// The planned position of the agents, not a server's actual state.
struct AgentCursor {
    /// Current position of the agent.
    pos: CellId,
    /// Amount of remaining fuels.
    fuel: u32,
    /// Number of steps whose plans are already confirmed.
    fixed_steps: u32,
}

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

        // Select spots for each agents
        let assignments = self.distribute_agents(day, &state, &cost_map);

        for (agent_id, spot) in assignments {
            // Create a route to the spot
            if let Some(cursor) = state.cursor.get_mut(agent_id) {
                let route = self.get_route(&cost_map, cursor.pos, spot);
                let Some(actions) = self.route_to_actions(&route) else {
                    break;
                };
                state.plan.extend_actions(agent_id, actions);
                cursor.pos = spot;
            }
        }

        self.fill_remaining_waits(day, &mut state);
        state.plan
    }
}
