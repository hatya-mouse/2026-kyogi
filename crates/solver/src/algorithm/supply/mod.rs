mod state;

use crate::{
    algorithm::{CostMap, PlanningState, Solver, supply::state::SupplyState},
    game::{Action, CellId, DayData},
};
use std::num::NonZeroU32;

struct RandezvousSpot {
    cell_id: CellId,
    patrol_arrival_steps: u32,
}

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

            for (agent_id, supply_agent) in supply_state.supplies.iter_mut() {
                let Some(assigned_patrol) = supply_agent
                    .current_patrol
                    .and_then(|agent_id| supply_agent.assigned_patrols.get(agent_id))
                else {
                    continue;
                };
                let Some(supply_cursor) = state.cursor.get(*agent_id) else {
                    continue;
                };
                let Some(patrol_cursor) = state.cursor.get(*assigned_patrol) else {
                    continue;
                };

                // Run dijkstra algorithm to get steps to each cells
                let cell_steps = self.dijkstra(cost_map, &supply_cursor.pos);

                // Search for possible randezvous spots
                let mut randezvous_spot = None;
                for (patrol_arrival_steps, patrol_pos) in patrol_cursor
                    .pos_history
                    .iter()
                    .enumerate()
                    .filter_map(|(steps, pos)| steps.try_into().map(|steps| (steps, pos)).ok())
                {
                    let steps_to_cell = cell_steps[patrol_pos.as_usize()];
                    let supply_arrival_steps = supply_cursor.fixed_steps + steps_to_cell;
                    if supply_arrival_steps <= patrol_arrival_steps {
                        randezvous_spot = Some(RandezvousSpot {
                            cell_id: *patrol_pos,
                            patrol_arrival_steps,
                        });
                        break;
                    }
                }

                if let Some(randezvous_spot) = randezvous_spot {
                    // Calculate a route using A* algorithm
                    let route =
                        self.get_route(cost_map, supply_cursor.pos, randezvous_spot.cell_id);
                    let Some(actions) = self.route_to_actions(&route) else {
                        break;
                    };

                    // Add a direct route to the patrol agent
                    added = added
                        || state.try_add_actions(
                            self.board.map,
                            day,
                            *agent_id,
                            randezvous_spot.cell_id,
                            actions.clone(),
                        );

                    // Wait for the patrol agent to arrive
                    if let Some(patrol_cursor) = state.cursor.get(*agent_id)
                        && let Some(wait_steps) = NonZeroU32::new(
                            randezvous_spot.patrol_arrival_steps - patrol_cursor.fixed_steps,
                        )
                        && let Some(supply_plan) = state.plan.actions.get_mut(*agent_id)
                    {
                        supply_plan.push(Action::Wait(wait_steps));
                    }
                }

                supply_agent.next_patrol();
            }

            if !added {
                break;
            }
        }
    }
}
