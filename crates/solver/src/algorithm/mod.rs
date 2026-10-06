mod fill_remaining;
mod route;
mod utils;

use crate::game::{AgentKind, CellId, DayData, DayPlan, Map};
use std::collections::HashSet;

/// A temporary plan state that is used during planning.
struct PlanningState {
    /// A plan that is now being constructed.
    plan: DayPlan,
    /// Current temporary state of the agents.
    cursor: Vec<AgentCursor>,
    /// ID of spots that have already been reserved by agents.
    reserved_spots: HashSet<CellId>,
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
            reserved_spots: HashSet::new(),
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
}

impl<'a> Solver<'a> {
    pub fn new(map: &'a Map) -> Self {
        Self { map }
    }

    /// Create a solve result for the day.
    pub fn solve_day(&self, day: &DayData) -> DayPlan {
        let mut state = PlanningState::from_day(day);

        for agent_id in 0..day.agents.len() {
            // For now skip supply agents
            if day.agents[agent_id].kind != AgentKind::Patrol {
                continue;
            }
        }

        self.fill_remaining_waits(day, &mut state);
        state.plan
    }
}
