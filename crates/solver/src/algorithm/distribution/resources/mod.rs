use crate::algorithm::planning_state::SpotState;
use crate::{
    algorithm::{CostMap, Solver, utils::get_cell_steps, utils::get_move_fuel},
    game::{CellId, DayData},
};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug)]
pub(super) struct RouteCost {
    pub steps: u32,
    pub patrol_fuel: u32,
}

#[derive(Clone, Copy, Debug, Default)]
pub(in crate::algorithm) struct FollowUpValue {
    pub new_brand: bool,
    pub collection: bool,
}

#[derive(Default)]
pub(in crate::algorithm) struct RouteCostCache {
    routes: HashMap<(CellId, CellId), Option<RouteCost>>,
}

impl Solver<'_> {
    pub(super) fn cached_route_cost(
        &self,
        day: &DayData,
        cost_map: &CostMap,
        cache: &mut RouteCostCache,
        source: CellId,
        destination: CellId,
    ) -> Option<RouteCost> {
        if let Some(route) = cache.routes.get(&(source, destination)) {
            return *route;
        }

        let route = self.get_route(cost_map, source, destination);
        let cost = self.measure_route(day, &route);
        cache.routes.insert((source, destination), cost);
        cost
    }

    fn measure_route(&self, day: &DayData, route: &[CellId]) -> Option<RouteCost> {
        if route.is_empty() {
            return None;
        }

        let mut cost = RouteCost {
            steps: 0,
            patrol_fuel: 0,
        };
        for pair in route.windows(2) {
            let move_steps = get_cell_steps(self.board.map, day, &pair[0]);
            let move_fuel = get_move_fuel(self.board.map, &pair[0]);
            if move_steps == u32::MAX || move_fuel == u32::MAX {
                return None;
            }

            cost.steps = cost.steps.saturating_add(move_steps);
            cost.patrol_fuel = cost.patrol_fuel.saturating_add(move_fuel);
        }

        Some(cost)
    }

    pub(in crate::algorithm) fn follow_up_value(
        &self,
        day: &DayData,
        cost_map: &CostMap,
        cache: &mut RouteCostCache,
        first_spot: CellId,
        remaining_steps: u32,
        remaining_fuel: u32,
        spots: &HashMap<CellId, SpotState>,
        visited_spots: &HashSet<CellId>,
        unvisited_brands: &HashSet<crate::game::Brand>,
    ) -> FollowUpValue {
        let mut value = FollowUpValue::default();
        let first_brand = spots.get(&first_spot).map(|spot| spot.brand);

        for (spot_id, spot) in spots {
            if *spot_id == first_spot || spot.stocks == 0 || visited_spots.contains(spot_id) {
                continue;
            }

            let Some(route) = self.cached_route_cost(day, cost_map, cache, first_spot, *spot_id)
            else {
                continue;
            };
            if route.steps > remaining_steps || route.patrol_fuel > remaining_fuel {
                continue;
            }

            value.collection = true;
            value.new_brand |=
                first_brand != Some(spot.brand) && unvisited_brands.contains(&spot.brand);
            if value.new_brand {
                break;
            }
        }

        value
    }
}
