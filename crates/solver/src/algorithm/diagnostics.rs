use crate::{
    algorithm::{AssignmentAttempt, distribution::Assignment, planning_state::PlanningState},
    game::{Action, AgentKind, CellId, DayData, Spot},
};
use std::{collections::HashMap, collections::HashSet};

#[derive(Default)]
pub(super) struct AllocationDiagnostics {
    pub(super) assignment_attempts: usize,
    pub(super) assignment_successes: usize,
    pub(super) route_failures: usize,
    pub(super) action_rejections: usize,
    pub(super) fuel_rejections: usize,
    pub(super) step_rejections: usize,
    pub(super) selected: Vec<(usize, CellId)>,
    pub(super) feasible_candidates_per_agent: HashMap<usize, usize>,
}

impl AllocationDiagnostics {
    pub(super) fn record_candidates(&mut self, counts: &[usize]) {
        for (agent_id, count) in counts.iter().enumerate() {
            self.feasible_candidates_per_agent.insert(agent_id, *count);
        }
    }

    pub(super) fn record_attempt(&mut self, result: AssignmentAttempt) {
        self.assignment_attempts += 1;
        match result {
            AssignmentAttempt::Added => self.assignment_successes += 1,
            AssignmentAttempt::RouteFailure => self.route_failures += 1,
            AssignmentAttempt::ActionRejected => self.action_rejections += 1,
            AssignmentAttempt::FuelRejected => self.fuel_rejections += 1,
            AssignmentAttempt::StepRejected => self.step_rejections += 1,
        }
    }

    pub(super) fn log_attempt(day: &DayData, assignment: &Assignment, result: &str) {
        eprintln!(
            "distribution day={} attempt agent={} spot={} brand={} result={}",
            day.day,
            assignment.agent_id,
            assignment.cell_id.as_usize(),
            assignment.brand.id(),
            result,
        );
    }

    pub(super) fn log_final(
        &self,
        day: &DayData,
        state: &PlanningState,
        spots: &HashMap<CellId, Spot>,
    ) {
        let mut assigned_brands = HashSet::new();
        let mut assigned_spots = 0usize;
        let mut idle_with_candidates = 0usize;
        let mut idle_without_candidates = 0usize;

        for (agent_id, agent) in day.agents.iter().enumerate() {
            if agent.kind != AgentKind::Patrol {
                continue;
            }

            let Some(cursor) = state.cursor.get(agent_id) else {
                continue;
            };
            let mut brands = HashSet::new();
            let mut movement_actions = 0usize;
            let mut wait_steps = 0u32;
            for action in state.plan.actions.get(agent_id).into_iter().flatten() {
                match action {
                    Action::Move(_) => movement_actions += 1,
                    Action::Wait(steps) => wait_steps += steps.get(),
                }
            }
            let movement_steps = day.steps.saturating_sub(wait_steps);
            for spot_id in &cursor.visited_spots {
                if let Some(spot) = spots.get(spot_id) {
                    brands.insert(*spot.brand());
                }
            }
            assigned_spots += cursor.visited_spots.len();
            assigned_brands.extend(brands.iter().copied());

            let selected_count = self
                .selected
                .iter()
                .filter(|(id, _)| *id == agent_id)
                .count();
            let status = if !cursor.visited_spots.is_empty() {
                "collected"
            } else if selected_count > 0 {
                "selected_but_not_in_final_cursor"
            } else if cursor.fixed_steps == day.steps {
                "day_steps_exhausted"
            } else if let Some(candidate_count) = self.feasible_candidates_per_agent.get(&agent_id)
            {
                if *candidate_count > 0 {
                    "idle_despite_reachable_within_day_candidate"
                } else {
                    "idle_no_reachable_within_day_candidate"
                }
            } else {
                "no_candidate_iteration_recorded"
            };
            if status == "idle_despite_reachable_within_day_candidate" {
                idle_with_candidates += 1;
            }
            if status == "idle_no_reachable_within_day_candidate" {
                idle_without_candidates += 1;
            }
            eprintln!(
                "distribution day={} final agent={} pos={} assigned_distinct_brands={} assigned_spots={} movement_actions={} movement_steps={} wait_steps={} remaining_fuel={} status={}",
                day.day,
                agent_id,
                cursor.pos.as_usize(),
                brands.len(),
                cursor.visited_spots.len(),
                movement_actions,
                movement_steps,
                wait_steps,
                cursor.fuel,
                status,
            );
        }

        eprintln!(
            "distribution day={} totals assigned_distinct_brands={} assigned_spots={} idle_with_reachable_candidate={} idle_without_reachable_candidate={} assignment_attempts={} assignment_successes={} route_failures={} fuel_rejections={} step_rejections={} other_rejections={}",
            day.day,
            assigned_brands.len(),
            assigned_spots,
            idle_with_candidates,
            idle_without_candidates,
            self.assignment_attempts,
            self.assignment_successes,
            self.route_failures,
            self.fuel_rejections,
            self.step_rejections,
            self.action_rejections,
        );
    }
}
