mod cost_map;
mod diagnostics;
mod distribution;
mod graph;
mod planning_state;
mod recalculation;
mod route;
mod supply;
mod utils;

#[cfg(test)]
mod tests;

use crate::{
    algorithm::{
        cost_map::CostMap,
        diagnostics::AllocationDiagnostics,
        distribution::{Assignment, RouteCostCache},
        graph::AdjGraph,
        planning_state::PlanningState,
        supply::SupplyState,
    },
    game::{AgentKind, Board, CellId, DayData, DayPlan, Map, Spot},
};
use std::collections::HashMap;

#[derive(Clone, Copy)]
pub(super) enum AssignmentAttempt {
    Added,
    RouteFailure,
    ActionRejected,
    FuelRejected,
    StepRejected,
}

/// A solver that calculates the plan for a day.
pub struct Solver<'a> {
    /// The board of the game.
    board: Board<'a>,
    /// Adjacent graph for the map.
    adj_graph: AdjGraph,
}

impl<'a> Solver<'a> {
    pub fn new(
        map: &'a Map,
        spots: HashMap<CellId, Spot>,
        agent_count: usize,
        fuel_limits: u32,
    ) -> Self {
        let adj_graph = AdjGraph::build(map);
        Self {
            board: Board::new(map, spots, agent_count, fuel_limits),
            adj_graph,
        }
    }

    /// Creates a solve result for the day.
    pub fn solve_day(&self, day: &DayData) -> DayPlan {
        let mut state = PlanningState::from_day(&self.board, day);
        let cost_map = CostMap::build(self.board.map, day);
        let mut route_costs = RouteCostCache::default();
        let diagnostics_enabled = std::env::var_os("SOLVER_DISTRIBUTION_DIAGNOSTICS").is_some();
        let mut diagnostics = AllocationDiagnostics::default();
        let mut excluded_assignments = std::collections::HashSet::new();

        loop {
            let (assignments, candidate_counts) = if diagnostics_enabled {
                self.distribute_agents_with_diagnostics(
                    day,
                    &state,
                    &cost_map,
                    &excluded_assignments,
                    &mut route_costs,
                )
            } else {
                (
                    self.distribute_agents_excluding(
                        day,
                        &state,
                        &cost_map,
                        &excluded_assignments,
                        &mut route_costs,
                    ),
                    Vec::new(),
                )
            };

            if diagnostics_enabled {
                diagnostics.record_candidates(
                    &candidate_counts
                        .iter()
                        .map(|count| count.feasible)
                        .collect::<Vec<_>>(),
                );
                distribution::log_candidate_summary(day, &state, &candidate_counts, &assignments);
            }

            if assignments.is_empty() || !state.has_remaining(day) {
                if diagnostics_enabled && assignments.is_empty() {
                    eprintln!("distribution day={} termination=no_candidates", day.day);
                }
                break;
            }

            let mut added = false;
            let mut rejected_assignment = false;
            for assignment in assignments {
                let attempt = self.try_add_assignment(&mut state, day, &cost_map, &assignment);
                if diagnostics_enabled {
                    diagnostics.record_attempt(attempt);
                }
                match attempt {
                    AssignmentAttempt::RouteFailure => {
                        if diagnostics_enabled {
                            AllocationDiagnostics::log_attempt(day, &assignment, "route_failure");
                        }
                        excluded_assignments.insert((assignment.agent_id, assignment.cell_id));
                        rejected_assignment = true;
                        break;
                    }
                    AssignmentAttempt::ActionRejected => {
                        if diagnostics_enabled {
                            AllocationDiagnostics::log_attempt(day, &assignment, "invalid_route");
                        }
                        excluded_assignments.insert((assignment.agent_id, assignment.cell_id));
                        rejected_assignment = true;
                        break;
                    }
                    AssignmentAttempt::FuelRejected => {
                        if diagnostics_enabled {
                            AllocationDiagnostics::log_attempt(day, &assignment, "fuel_rejected");
                        }
                        excluded_assignments.insert((assignment.agent_id, assignment.cell_id));
                        rejected_assignment = true;
                        break;
                    }
                    AssignmentAttempt::StepRejected => {
                        if diagnostics_enabled {
                            AllocationDiagnostics::log_attempt(day, &assignment, "step_rejected");
                        }
                        excluded_assignments.insert((assignment.agent_id, assignment.cell_id));
                        rejected_assignment = true;
                        break;
                    }
                    AssignmentAttempt::Added => {}
                }

                self.visited_spot(&mut state, assignment.agent_id, assignment.cell_id);
                if diagnostics_enabled {
                    diagnostics
                        .selected
                        .push((assignment.agent_id, assignment.cell_id));
                    AllocationDiagnostics::log_attempt(day, &assignment, "selected");
                }
                added = true;
                break;
            }

            if added {
                excluded_assignments.clear();
            }

            if !added && !rejected_assignment {
                if diagnostics_enabled {
                    eprintln!(
                        "distribution day={} termination=all_candidates_rejected",
                        day.day
                    );
                }
                break;
            }
        }

        for agent_id in day
            .agents
            .iter()
            .enumerate()
            .filter(|(_, agent)| agent.kind == AgentKind::Patrol)
            .map(|(id, _)| id)
        {
            state.fill_remaining_waits(day, agent_id);
        }

        let mut supply_state = SupplyState::new(day);
        self.build_supply_plan(&mut state, &mut supply_state, day, &cost_map);
        self.recalculate_after_refills(&mut state, &supply_state, day, &cost_map, &mut route_costs);

        for agent_id in day
            .agents
            .iter()
            .enumerate()
            .filter(|(_, agent)| agent.kind == AgentKind::Supply)
            .map(|(id, _)| id)
        {
            state.fill_remaining_waits(day, agent_id);
        }

        if diagnostics_enabled {
            diagnostics.log_final(day, &state, &self.board.spots);
        }
        state.plan
    }

    /// Tries to add a direct route or a synchronized supply route.
    fn try_add_assignment(
        &self,
        state: &mut PlanningState,
        day: &DayData,
        cost_map: &CostMap,
        assignment: &Assignment,
    ) -> AssignmentAttempt {
        let Some(cursor) = state.cursor.get(assignment.agent_id).cloned() else {
            return AssignmentAttempt::ActionRejected;
        };

        let route = self.get_route(cost_map, cursor.pos, assignment.cell_id);
        let Some(actions) = self.route_to_actions(&route) else {
            return AssignmentAttempt::RouteFailure;
        };

        match state.try_add_actions_detailed(
            self.board.map,
            day,
            assignment.agent_id,
            assignment.cell_id,
            actions,
        ) {
            planning_state::ActionInsertResult::Added => AssignmentAttempt::Added,
            planning_state::ActionInsertResult::StepRejected => AssignmentAttempt::StepRejected,
            planning_state::ActionInsertResult::FuelRejected => AssignmentAttempt::FuelRejected,
            planning_state::ActionInsertResult::Invalid => AssignmentAttempt::ActionRejected,
        }
    }

    fn visited_spot(&self, state: &mut PlanningState, agent_id: usize, spot_id: CellId) {
        if let Some(spot) = self.board.spots.get(&spot_id) {
            state.unvisited_brands.remove(spot.brand());
        }
        if let Some(spot_state) = state.spots.get_mut(&spot_id) {
            spot_state.stocks = spot_state.stocks.saturating_sub(1);
        }
        if let Some(cursor) = state.cursor.get_mut(agent_id) {
            cursor.visited_spots.insert(spot_id);
        }
    }
}
