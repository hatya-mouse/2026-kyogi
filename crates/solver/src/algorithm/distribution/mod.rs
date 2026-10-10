//! Distribute agents using a simple ranking of reachable spots.

mod dijkstra;
mod matching;
mod prioritization;
mod resources;

use crate::{
    algorithm::{CostMap, PlanningState, Solver, planning_state::SpotState},
    game::{AgentKind, Brand, CellId, DayData},
};
use std::{collections::HashSet, env};

use matching::allocate_brand_coverage;
use prioritization::{limit_candidates_by_stock, prioritize_candidates};
pub(super) use resources::RouteCostCache;

struct SpotCandidate {
    pub(super) agent_id: usize,
    pub(super) spot_id: CellId,
    pub(super) steps: u32,
    pub(super) slack: u32,
    pub(super) fuel: u32,
    pub(super) lookahead_brand: bool,
    pub(super) lookahead_collection: bool,
}

#[derive(Debug, Clone)]
pub(super) struct Assignment {
    pub(super) agent_id: usize,
    pub(super) cell_id: CellId,
    pub(super) brand: Brand,
}

impl Assignment {
    fn from_candidate(candidate: SpotCandidate, brand: Brand) -> Self {
        Self {
            agent_id: candidate.agent_id,
            cell_id: candidate.spot_id,
            brand,
        }
    }
}

#[derive(Default, Clone)]
pub(super) struct CandidateCounts {
    pub(super) occupied: usize,
    pub(super) empty_stock: usize,
    pub(super) visited_spot: usize,
    pub(super) unreachable: usize,
    pub(super) exceeds_day: usize,
    pub(super) fuel_infeasible: usize,
    pub(super) feasible: usize,
    pub(super) capacity_pruned: usize,
    pub(super) reachable_unvisited_brands: HashSet<Brand>,
}

impl Solver<'_> {
    pub(super) fn distribute_agents_excluding(
        &self,
        day: &DayData,
        state: &PlanningState,
        cost_map: &CostMap,
        excluded: &HashSet<(usize, CellId)>,
        route_costs: &mut RouteCostCache,
    ) -> Vec<Assignment> {
        self.build_candidates(day, state, cost_map, false, excluded, None, route_costs)
            .0
    }

    pub(super) fn distribute_patrol(
        &self,
        day: &DayData,
        state: &PlanningState,
        cost_map: &CostMap,
        patrol_id: usize,
        excluded: &HashSet<(usize, CellId)>,
        route_costs: &mut RouteCostCache,
    ) -> Vec<Assignment> {
        self.build_candidates(
            day,
            state,
            cost_map,
            false,
            excluded,
            Some(patrol_id),
            route_costs,
        )
        .0
    }

    pub(super) fn distribute_agents_with_diagnostics(
        &self,
        day: &DayData,
        state: &PlanningState,
        cost_map: &CostMap,
        excluded: &HashSet<(usize, CellId)>,
        route_costs: &mut RouteCostCache,
    ) -> (Vec<Assignment>, Vec<CandidateCounts>) {
        self.build_candidates(day, state, cost_map, true, excluded, None, route_costs)
    }

    fn build_candidates(
        &self,
        day: &DayData,
        state: &PlanningState,
        cost_map: &CostMap,
        collect_diagnostics: bool,
        excluded: &HashSet<(usize, CellId)>,
        only_agent: Option<usize>,
        route_costs: &mut RouteCostCache,
    ) -> (Vec<Assignment>, Vec<CandidateCounts>) {
        let mut candidates = Vec::new();
        let mut counts =
            collect_diagnostics.then(|| vec![CandidateCounts::default(); day.agents.len()]);
        for (agent_id, (agent, cursor)) in day.agents.iter().zip(state.cursor.iter()).enumerate() {
            if agent.kind != AgentKind::Patrol || only_agent.is_some_and(|only| only != agent_id) {
                continue;
            }

            for (spot_id, SpotState { stocks, .. }) in &state.spots {
                if *spot_id == cursor.pos {
                    if let Some(count) = counts.as_mut().and_then(|counts| counts.get_mut(agent_id))
                    {
                        count.occupied += 1;
                    }
                    continue;
                }
                if *stocks == 0 {
                    if let Some(count) = counts.as_mut().and_then(|counts| counts.get_mut(agent_id))
                    {
                        count.empty_stock += 1;
                    }
                    continue;
                }
                if cursor.visited_spots.contains(spot_id) {
                    if let Some(count) = counts.as_mut().and_then(|counts| counts.get_mut(agent_id))
                    {
                        count.visited_spot += 1;
                    }
                    continue;
                }

                let Some(route_cost) =
                    self.cached_route_cost(day, cost_map, route_costs, cursor.pos, *spot_id)
                else {
                    if let Some(count) = counts.as_mut().and_then(|counts| counts.get_mut(agent_id))
                    {
                        count.unreachable += 1;
                    }
                    continue;
                };
                let arrival_step = cursor.fixed_steps.saturating_add(route_cost.steps);
                if arrival_step > day.steps {
                    if let Some(count) = counts.as_mut().and_then(|counts| counts.get_mut(agent_id))
                    {
                        count.exceeds_day += 1;
                    }
                    continue;
                }
                if agent.kind == AgentKind::Patrol && route_cost.patrol_fuel > cursor.fuel {
                    if let Some(count) = counts.as_mut().and_then(|counts| counts.get_mut(agent_id))
                    {
                        count.fuel_infeasible += 1;
                    }
                    continue;
                }
                if let Some(count) = counts.as_mut().and_then(|counts| counts.get_mut(agent_id)) {
                    count.feasible += 1;
                    let brand = state.spots[spot_id].brand;
                    if state.unvisited_brands.contains(&brand) {
                        count.reachable_unvisited_brands.insert(brand);
                    }
                }

                let follow_up = self.follow_up_value(
                    day,
                    cost_map,
                    route_costs,
                    *spot_id,
                    day.steps.saturating_sub(arrival_step),
                    cursor.fuel.saturating_sub(route_cost.patrol_fuel),
                    &state.spots,
                    &cursor.visited_spots,
                    &state.unvisited_brands,
                );
                candidates.push(SpotCandidate {
                    agent_id,
                    spot_id: *spot_id,
                    steps: route_cost.steps,
                    slack: day.steps.saturating_sub(arrival_step),
                    fuel: route_cost.patrol_fuel,
                    lookahead_brand: follow_up.new_brand,
                    lookahead_collection: follow_up.collection,
                });
            }
        }

        prioritize_candidates(state, &mut candidates);
        candidates.retain(|candidate| !excluded.contains(&(candidate.agent_id, candidate.spot_id)));

        let brand_coverage_enabled = env::var("SOLVER_BRAND_COVERAGE").as_deref() != Ok("0");
        if brand_coverage_enabled {
            candidates = allocate_brand_coverage(state, candidates);
        }

        let candidates_before_stock_limit = if counts.is_some() {
            let mut by_agent = vec![0usize; day.agents.len()];
            for candidate in &candidates {
                by_agent[candidate.agent_id] += 1;
            }
            Some(by_agent)
        } else {
            None
        };
        limit_candidates_by_stock(state, &mut candidates);

        if let (Some(counts), Some(before)) = (counts.as_mut(), candidates_before_stock_limit) {
            let mut retained_by_agent = vec![0usize; day.agents.len()];
            for candidate in &candidates {
                retained_by_agent[candidate.agent_id] += 1;
            }
            for agent_id in 0..counts.len() {
                counts[agent_id].capacity_pruned = before[agent_id] - retained_by_agent[agent_id];
            }
        }

        let assignments = candidates
            .into_iter()
            .filter_map(|candidate| {
                state
                    .spots
                    .get(&candidate.spot_id)
                    .map(|spot| Assignment::from_candidate(candidate, spot.brand))
            })
            .collect();
        (assignments, counts.unwrap_or_default())
    }
}

pub(super) fn log_candidate_summary(
    day: &DayData,
    state: &PlanningState,
    counts: &[CandidateCounts],
    candidates: &[Assignment],
) {
    if std::env::var_os("SOLVER_DISTRIBUTION_DIAGNOSTICS").is_none() {
        return;
    }

    let mut retained_spots = vec![Vec::new(); day.agents.len()];
    for candidate in candidates {
        retained_spots[candidate.agent_id].push(candidate.cell_id);
    }

    for (agent_id, agent) in day.agents.iter().enumerate() {
        if agent.kind != AgentKind::Patrol {
            continue;
        }
        let Some(cursor) = state.cursor.get(agent_id) else {
            continue;
        };
        let Some(count) = counts.get(agent_id) else {
            continue;
        };

        eprintln!(
            "distribution day={} agent={} pos={} fixed_steps={} remaining_steps={} fuel={} candidates={{occupied:{}, empty_stock:{}, visited_spot:{}, unreachable:{}, exceeds_day:{}, fuel_infeasible:{}, reachable_within_day:{}, capacity_pruned:{}, retained_after_stock:{}}} reachable_unvisited_brands={:?} retained_spots={:?}",
            day.day,
            agent_id,
            cursor.pos.as_usize(),
            cursor.fixed_steps,
            day.steps.saturating_sub(cursor.fixed_steps),
            cursor.fuel,
            count.occupied,
            count.empty_stock,
            count.visited_spot,
            count.unreachable,
            count.exceeds_day,
            count.fuel_infeasible,
            count.feasible,
            count.capacity_pruned,
            retained_spots.get(agent_id).map_or(0, Vec::len),
            count.reachable_unvisited_brands,
            retained_spots.get(agent_id),
        );
    }
}
