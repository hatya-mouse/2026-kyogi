use crate::{
    algorithm::{CostMap, PlanningState, Solver, distribution::Assignment},
    game::DayData,
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
        state.try_add_actions(
            self.board.map,
            day,
            assignment.agent_id,
            assignment.spot_id,
            actions.clone(),
        )
    }
}
