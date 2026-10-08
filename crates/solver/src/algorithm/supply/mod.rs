mod state;

use crate::{
    algorithm::{CostMap, PlanningState, Solver, supply::state::SupplyState},
    game::DayData,
};

impl Solver<'_> {
    pub(super) fn build_supply_plan(
        &self,
        state: &mut PlanningState,
        day: &DayData,
        cost_map: &CostMap,
    ) {
        let mut supply_state = SupplyState::new(day);
        supply_state.reset_current_patrols();

        loop {
            let mut added = false;

            for (agent_id, cursor) in supply_state.supplies.iter_mut() {
                let Some(assigned_patrol) = cursor
                    .current_patrol
                    .and_then(|agent_id| cursor.assigned_patrols.get(agent_id))
                else {
                    continue;
                };
                let Some(patrol_cursor) = state.cursor.get(*assigned_patrol) else {
                    continue;
                };

                // Run dijkstra algorithm to get steps to each cells
                let cell_steps = self.dijkstra(cost_map, &cursor.pos);

                // Search for possible randezvous spots
                let mut randezvous_spot = None;
                for (patrol_arrival_steps, patrol_pos) in patrol_cursor
                    .pos_history
                    .iter()
                    .enumerate()
                    .filter_map(|(steps, pos)| steps.try_into().map(|steps| (steps, pos)).ok())
                {
                    let steps_to_cell = cell_steps[patrol_pos.as_usize()];
                    let supply_arrival_steps = cursor.fixed_steps + steps_to_cell;
                    if supply_arrival_steps <= patrol_arrival_steps {
                        randezvous_spot = Some(*patrol_pos);
                        break;
                    }
                }

                if let Some(randezvous_spot) = randezvous_spot {
                    // Calculate a route using A* algorithm
                    let route = self.get_route(cost_map, cursor.pos, randezvous_spot);
                    let Some(actions) = self.route_to_actions(&route) else {
                        break;
                    };

                    // Add a direct route to the patrol car
                    added = added
                        || state.try_add_actions(
                            self.board.map,
                            day,
                            *agent_id,
                            randezvous_spot,
                            actions.clone(),
                        );
                }

                cursor.next_patrol();
            }

            if !added {
                break;
            }
        }
    }
}
