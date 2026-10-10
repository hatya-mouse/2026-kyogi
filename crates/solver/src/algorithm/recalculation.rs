use crate::{
    algorithm::{CostMap, PlanningState, Solver, supply::SupplyState, utils::get_action_steps},
    game::{Action, Brand, CellId, DayData},
};
use std::collections::HashSet;

impl Solver<'_> {
    pub(super) fn recalculate_after_refills(
        &self,
        state: &mut PlanningState,
        supply_state: &SupplyState,
        day: &DayData,
        cost_map: &CostMap,
    ) {
        let mut refills = supply_state.refills.clone();
        refills.sort_unstable_by_key(|refill| refill.step);
        let mut recalculated_patrols = HashSet::new();

        for refill in refills {
            if !recalculated_patrols.insert(refill.patrol_id) {
                continue;
            }

            self.restore_patrol_state(state, refill.patrol_id, refill.step);
            if !self.truncate_patrol_plan(state, day, refill.patrol_id, refill.cell_id, refill.step)
            {
                continue;
            }
            self.recalculate_patrol(state, day, cost_map, refill.patrol_id);
        }
    }

    fn restore_patrol_state(&self, state: &mut PlanningState, patrol_id: usize, refill_step: u32) {
        let Some(cursor) = state.cursor.get(patrol_id).cloned() else {
            return;
        };

        let removed_spots: Vec<CellId> = cursor
            .visited_spots
            .into_iter()
            .filter(|spot_id| {
                cursor
                    .pos_history
                    .iter()
                    .enumerate()
                    .any(|(step, position)| step as u32 > refill_step && *position == *spot_id)
            })
            .collect();

        for spot_id in removed_spots {
            if let Some(spot) = state.spots.get_mut(&spot_id) {
                spot.stocks = spot.stocks.saturating_add(1);
            }

            if let Some(cursor) = state.cursor.get_mut(patrol_id) {
                cursor.visited_spots.remove(&spot_id);
            }
        }

        state.unvisited_brands = self.unvisited_brands(state);
    }

    fn unvisited_brands(&self, state: &PlanningState) -> std::collections::HashSet<Brand> {
        let visited_brands: std::collections::HashSet<Brand> = state
            .cursor
            .iter()
            .flat_map(|cursor| cursor.visited_spots.iter())
            .filter_map(|spot_id| state.spots.get(spot_id))
            .map(|spot| spot.brand)
            .collect();

        self.board
            .brands
            .keys()
            .filter(|brand| !visited_brands.contains(brand))
            .copied()
            .collect()
    }

    fn truncate_patrol_plan(
        &self,
        state: &mut PlanningState,
        day: &DayData,
        patrol_id: usize,
        refill_cell: CellId,
        refill_step: u32,
    ) -> bool {
        let Some(cursor) = state.cursor.get(patrol_id).cloned() else {
            return false;
        };
        let Some(actions) = state.plan.actions.get(patrol_id).cloned() else {
            return false;
        };

        let mut steps: u32 = 0;
        let mut position = cursor.pos_history[0];
        let mut action_count = 0;

        for action in actions {
            let mut action_cursor = cursor.clone();
            action_cursor.pos = position;
            let action_steps = get_action_steps(self.board.map, day, &action_cursor, &action);
            if steps.saturating_add(action_steps) > refill_step {
                break;
            }

            if let Action::Move(direction) = action {
                let coord = self.board.map.get_coord_from_id(position);
                position = self
                    .board
                    .map
                    .get_id_from_coord(direction.apply_to_coord(coord));
            }

            steps += action_steps;
            action_count += 1;
        }

        if position != refill_cell || steps != refill_step {
            return false;
        }

        if let Some(actions) = state.plan.actions.get_mut(patrol_id) {
            actions.truncate(action_count);
        }

        if let Some(cursor) = state.cursor.get_mut(patrol_id) {
            cursor.fixed_steps = steps;
            cursor.fuel = self.board.fuel_limits;
            cursor.pos_history.truncate(steps as usize + 1);
            if let Some(&position) = cursor.pos_history.last() {
                cursor.pos = position;
            }
        }

        true
    }

    fn recalculate_patrol(
        &self,
        state: &mut PlanningState,
        day: &DayData,
        cost_map: &CostMap,
        patrol_id: usize,
    ) {
        let mut excluded_assignments = HashSet::new();
        loop {
            let assignments =
                self.distribute_patrol(day, state, cost_map, patrol_id, &excluded_assignments);

            let mut added = false;
            let mut failed = false;
            for assignment in assignments {
                if assignment.agent_id != patrol_id {
                    continue;
                }
                if !matches!(
                    self.try_add_assignment(state, day, cost_map, &assignment),
                    super::AssignmentAttempt::Added
                ) {
                    excluded_assignments.insert((assignment.agent_id, assignment.cell_id));
                    failed = true;
                    break;
                }

                self.visited_spot(state, patrol_id, assignment.cell_id);
                excluded_assignments.clear();
                added = true;
                break;
            }

            if !added && !failed {
                break;
            }
        }

        state.fill_remaining_waits(day, patrol_id);
    }
}
