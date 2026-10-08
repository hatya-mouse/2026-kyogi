use crate::{
    algorithm::utils::{get_action_steps, get_move_fuel},
    game::{Action, AgentKind, Board, Brand, CellId, DayData, DayPlan, Map, Spot},
};
use std::{
    collections::{HashMap, HashSet},
    num::NonZeroU32,
};

/// A temporary plan state that is used during planning.
/// Will be reset every day.
#[derive(Clone, Debug)]
pub(super) struct PlanningState {
    /// A plan that is now being constructed.
    pub plan: DayPlan,
    /// Current temporary state of the agents.
    pub cursor: Vec<AgentCursor>,
    /// Current temporary state of the spots.
    pub spots: HashMap<CellId, SpotState>,
    /// Set of brands that has not been visited today.
    pub unvisited_brands: HashSet<Brand>,
    /// IDs of the agents assigned as supply cars.
    pub supply_agents: HashSet<usize>,
}

/// The planned position of the agents, not a server's actual state.
#[derive(Clone, Debug)]
pub(super) struct AgentCursor {
    /// Current position of the agent.
    pub pos: CellId,
    /// Amount of remaining fuels.
    pub fuel: u32,
    /// Number of steps whose plans are already confirmed.
    pub fixed_steps: u32,
    /// Spots that this agent has visited in the day.
    pub visited_spots: HashSet<CellId>,
    /// Position of the agent at the start of the steps.
    pub pos_history: Vec<CellId>,
}

impl AgentCursor {
    fn new(pos: CellId, fuel: u32) -> Self {
        Self {
            pos,
            fuel,
            fixed_steps: 0,
            visited_spots: HashSet::new(),
            pos_history: vec![pos],
        }
    }
}

/// The estimated state of the spot.
#[derive(Clone, Debug)]
pub(super) struct SpotState {
    /// Brand of the spot.
    pub brand: Brand,
    /// Current stocks of the spot.
    pub stocks: u32,
}

impl SpotState {
    fn from_spot(spot: &Spot) -> Self {
        Self {
            brand: *spot.brand(),
            stocks: spot.stocks(),
        }
    }
}

impl PlanningState {
    /// Creates a new planning state from day data.
    pub(super) fn from_day(board: &Board, day: &DayData) -> Self {
        let agent_count = day.agents.len();

        Self {
            plan: DayPlan::new(agent_count),
            cursor: day
                .agents
                .iter()
                .map(|agent| AgentCursor::new(agent.pos, agent.fuel))
                .collect(),
            unvisited_brands: board.brands.keys().copied().collect(),
            spots: board
                .spots
                .iter()
                .map(|(id, spot)| (*id, SpotState::from_spot(spot)))
                .collect(),
            supply_agents: day
                .agents
                .iter()
                .enumerate()
                .filter_map(|(agent_id, agent)| {
                    (agent.kind == AgentKind::Supply).then_some(agent_id)
                })
                .collect(),
        }
    }

    /// Returns whether the current plan has remaining steps for any agents.
    pub(super) fn has_remaining(&self, day: &DayData) -> bool {
        for cursor in &self.cursor {
            let remaining_steps = day.steps.saturating_sub(cursor.fixed_steps);
            if remaining_steps > 0 {
                return true;
            }
        }
        false
    }

    pub(super) fn try_add_actions(
        &mut self,
        map: &Map,
        day: &DayData,
        agent_id: usize,
        destination: CellId,
        actions: Vec<Action>,
    ) -> bool {
        let Some(cursor) = self.cursor.get_mut(agent_id) else {
            return false;
        };
        let Some(agent) = day.agents.get(agent_id) else {
            return false;
        };
        let mut next_cursor = cursor.clone();
        let mut fuel_used: u32 = 0;
        let mut action_steps: u32 = 0;
        let mut next_pos_history = cursor.pos_history.clone();

        for action in &actions {
            let origin = next_cursor.pos;
            let steps = get_action_steps(map, day, &next_cursor, action);

            if let Action::Move(direction) = action {
                if !self.supply_agents.contains(&agent_id) {
                    fuel_used = fuel_used.saturating_add(get_move_fuel(map, &next_cursor.pos));
                }
                let coord = map.get_coord_from_id(next_cursor.pos);
                next_cursor.pos = map.get_id_from_coord(direction.apply_to_coord(coord));
            }

            append_position_history(
                &mut next_pos_history,
                origin,
                next_cursor.pos,
                action,
                steps,
            );
            action_steps += steps;
        }

        let exceeds_day_steps = cursor.fixed_steps.saturating_add(action_steps) > day.steps;
        let exceeds_fuel = fuel_used > cursor.fuel;
        let reaches_destination = next_cursor.pos == destination;

        if exceeds_day_steps || exceeds_fuel || !reaches_destination {
            return false;
        }

        self.plan.extend_actions(agent_id, actions);
        cursor.fixed_steps += action_steps;
        if agent.kind == AgentKind::Patrol {
            cursor.fuel -= fuel_used;
        }
        cursor.pos = destination;
        cursor.pos_history = next_pos_history;
        true
    }

    /// Pads each agent's unfinished plan with a wait action.
    /// Number of agents and the length of `actions` vector in the plan must be the same.
    /// If not, this function panics.
    ///
    /// # Arguments
    /// - `day`: The day data for the current game day.
    pub(super) fn fill_remaining_waits(&mut self, day: &DayData, agent_id: usize) {
        if let Some(agent_actions) = self.plan.actions.get_mut(agent_id)
            && let Some(cursor) = self.cursor.get_mut(agent_id)
        {
            let remaining_steps = day.steps.saturating_sub(cursor.fixed_steps);

            // Add wait instruction if the remaining steps is more than 0
            if let Some(wait_steps) = NonZeroU32::new(remaining_steps) {
                agent_actions.push(Action::Wait(wait_steps));

                cursor.fixed_steps = day.steps;
                cursor
                    .pos_history
                    .extend(vec![cursor.pos; remaining_steps as usize]);
            }
        }
    }
}

/// Appends the position after each effective step of an action.
fn append_position_history(
    history: &mut Vec<CellId>,
    origin: CellId,
    destination: CellId,
    action: &Action,
    steps: u32,
) {
    match action {
        Action::Move(_) => {
            if steps > 1 {
                history.extend(std::iter::repeat_n(origin, steps as usize - 1));
            }
            history.push(destination);
        }
        Action::Wait(_) => {
            history.extend(std::iter::repeat_n(destination, steps as usize));
        }
    }
}
