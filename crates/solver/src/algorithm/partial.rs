//! Moves patrol agents toward useful spots when a full assignment no longer fits.

use crate::{
    algorithm::{
        Solver,
        cost_map::CostMap,
        planning_state::PlanningState,
        utils::{get_action_steps, get_move_fuel},
    },
    game::{Action, CellId, DayData},
};
use std::cmp::Reverse;

struct PartialCandidate {
    destination: CellId,
    actions: Vec<Action>,
    steps: u32,
    unvisited_brand: bool,
}

impl Solver<'_> {
    /// Adds at most one partial route for the given patrol agent.
    pub(super) fn add_partial_movements(
        &self,
        state: &mut PlanningState,
        day: &DayData,
        cost_map: &CostMap,
        agent_id: usize,
    ) {
        // A partial route improves the next day's starting position without claiming a spot
        let Some(candidate) = self.select_partial_candidate(state, day, cost_map, agent_id) else {
            return;
        };

        state.try_add_actions(
            self.board.map,
            day,
            agent_id,
            candidate.destination,
            candidate.actions,
        );
    }

    fn select_partial_candidate(
        &self,
        state: &PlanningState,
        day: &DayData,
        cost_map: &CostMap,
        agent_id: usize,
    ) -> Option<PartialCandidate> {
        let cursor = state.cursor.get(agent_id)?;
        let mut candidates = Vec::new();

        for (spot_id, spot) in &state.spots {
            // Only consider spots that can still provide a useful future assignment
            if spot.stocks == 0 || cursor.visited_spots.contains(spot_id) || *spot_id == cursor.pos
            {
                continue;
            }

            // Build the complete route first, then keep only the prefix that fits today
            let route = self.get_route(cost_map, cursor.pos, *spot_id);
            let Some(candidate) = self.build_partial_candidate(
                state,
                day,
                agent_id,
                route,
                state.unvisited_brands.contains(&spot.brand),
            ) else {
                continue;
            };

            candidates.push(candidate);
        }

        // Prefer a new brand, then prefer the candidate that moves the agent farther
        candidates
            .into_iter()
            .min_by_key(|candidate| (!candidate.unvisited_brand, Reverse(candidate.steps)))
    }

    fn build_partial_candidate(
        &self,
        state: &PlanningState,
        day: &DayData,
        agent_id: usize,
        route: Vec<CellId>,
        unvisited_brand: bool,
    ) -> Option<PartialCandidate> {
        // A route with fewer than two cells cannot produce a movement action
        if route.len() < 2 {
            return None;
        }

        let cursor = state.cursor.get(agent_id)?;
        let remaining_steps = day.steps.saturating_sub(cursor.fixed_steps);
        let mut simulated_cursor = cursor.clone();
        let mut actions = Vec::new();
        let mut steps: u32 = 0;

        for pair in route.windows(2) {
            // Simulate each movement so traffic-dependent steps and fuel remain accurate
            let direction = self.board.map.direction_to(pair[0], pair[1])?;
            let action = Action::Move(direction);
            let action_steps = get_action_steps(self.board.map, day, &simulated_cursor, &action);
            let fuel = get_move_fuel(self.board.map, &simulated_cursor.pos);

            // Stop before the first movement that would exceed today's resources
            if steps.saturating_add(action_steps) > remaining_steps || fuel > simulated_cursor.fuel
            {
                break;
            }

            actions.push(action);
            steps = steps.saturating_add(action_steps);
            simulated_cursor.fixed_steps =
                simulated_cursor.fixed_steps.saturating_add(action_steps);
            simulated_cursor.fuel -= fuel;
            simulated_cursor.pos = pair[1];
        }

        // Do not add a no-op candidate when the first movement cannot fit
        if actions.is_empty() {
            return None;
        }

        // The spot is intentionally not marked as visited until the agent reaches it fully
        Some(PartialCandidate {
            destination: simulated_cursor.pos,
            actions,
            steps,
            unvisited_brand,
        })
    }
}
