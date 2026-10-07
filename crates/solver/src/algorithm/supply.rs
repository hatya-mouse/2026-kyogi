use std::num::NonZeroU32;

use crate::{
    algorithm::{
        CostMap, PlanningState, Solver, distribution::Assignment, planning_state::AgentCursor,
        utils::get_move_fuel,
    },
    game::{Action, CellId, DayData},
};

impl Solver<'_> {
    /// Tries to add a direct route or a synchronized supply route
    pub(super) fn try_add_assignment(
        &self,
        state: &mut PlanningState,
        day: &DayData,
        cost_map: &CostMap,
        assignment: &Assignment,
    ) -> bool {
        let Some(cursor) = state.cursor.get(assignment.agent_id).cloned() else {
            return false;
        };

        // Calculate a route using A* algorithm
        let route = self.get_route(cost_map, cursor.pos, assignment.spot_id);
        let Some(actions) = self.route_to_actions(&route) else {
            return false;
        };

        // Add a direct route to the patrol car
        if state.try_add_actions(
            self.board.map,
            day,
            assignment.agent_id,
            assignment.spot_id,
            actions.clone(),
        ) {
            return true;
        }

        // Try to rendezvous with a supply car when the direct route fails
        self.try_add_supply_route(state, day, cost_map, assignment, &route, cursor)
    }

    /// Adds a route with one supply-car rendezvous
    fn try_add_supply_route(
        &self,
        state: &mut PlanningState,
        day: &DayData,
        cost_map: &CostMap,
        assignment: &Assignment,
        route: &[CellId],
        cursor: AgentCursor,
    ) -> bool {
        let mut patrol_fuel = cursor.fuel;
        let mut patrol_steps = cursor.fixed_steps;

        for (index, pair) in route.windows(2).enumerate() {
            // Check the patrol car state at the next route cell
            patrol_fuel = patrol_fuel.saturating_sub(get_move_fuel(self.board.map, &pair[0]));
            patrol_steps =
                patrol_steps.saturating_add(cost_map.steps(&pair[0]).unwrap_or(u32::MAX));

            if patrol_fuel > 0 {
                continue;
            }

            // Try each supply car at the first fuel shortage
            for supply_id in state.supply_agents.clone() {
                let Some(supply) = state.cursor.get(supply_id).cloned() else {
                    continue;
                };

                let rendezvous = route[index + 1];
                let supply_route = self.get_route(cost_map, supply.pos, rendezvous);
                let Some(supply_actions) = self.route_to_actions(&supply_route) else {
                    continue;
                };

                let supply_steps = self.route_steps(self.board.map, day, &supply, &supply_actions);
                let supply_arrival = supply.fixed_steps.saturating_add(supply_steps);
                let patrol_arrival = patrol_steps;

                if supply_arrival > day.steps {
                    continue;
                }

                // Split the patrol route around the rendezvous cell
                let patrol_prefix = &route[..=index + 1];
                let Some(prefix_actions) = self.route_to_actions(patrol_prefix) else {
                    continue;
                };

                let suffix = &route[index + 1..];
                let Some(suffix_actions) = self.route_to_actions(suffix) else {
                    continue;
                };

                // Apply all actions to a temporary state before committing them
                let mut next_state = state.clone();

                if !next_state.try_add_actions(
                    self.board.map,
                    day,
                    supply_id,
                    rendezvous,
                    supply_actions,
                ) {
                    continue;
                }

                if !next_state.try_add_actions(
                    self.board.map,
                    day,
                    assignment.agent_id,
                    rendezvous,
                    prefix_actions,
                ) {
                    continue;
                }

                let wait_steps = supply_arrival.saturating_sub(patrol_arrival);
                if let Some(wait) = NonZeroU32::new(wait_steps)
                    && !next_state.try_add_actions(
                        self.board.map,
                        day,
                        assignment.agent_id,
                        rendezvous,
                        vec![Action::Wait(wait)],
                    )
                {
                    continue;
                }

                // Refill the patrol car when both cars meet
                if let Some(cursor) = next_state.cursor.get_mut(assignment.agent_id) {
                    cursor.fuel = state.fuel_limit;
                }

                if !next_state.try_add_actions(
                    self.board.map,
                    day,
                    assignment.agent_id,
                    assignment.spot_id,
                    suffix_actions,
                ) {
                    continue;
                }

                *state = next_state;
                return true;
            }
        }

        false
    }
}
