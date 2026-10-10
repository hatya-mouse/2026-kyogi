//! Distribute agents using a simple ranking of reachable spots.

mod dijkstra;
mod matching;

use crate::{
    algorithm::{CostMap, PlanningState, Solver, planning_state::SpotState},
    game::{AgentKind, Brand, CellId, DayData},
};
use std::{
    collections::{HashMap, HashSet, hash_map::Entry},
    env,
};

use matching::allocate_brand_coverage;

struct SpotCandidate {
    pub(super) agent_id: usize,
    pub(super) spot_id: CellId,
    pub(super) steps: u32,
    pub(super) slack: u32,
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
    ) -> Vec<Assignment> {
        self.build_candidates(day, state, cost_map, false, excluded, None)
            .0
    }

    pub(super) fn distribute_patrol(
        &self,
        day: &DayData,
        state: &PlanningState,
        cost_map: &CostMap,
        patrol_id: usize,
        excluded: &HashSet<(usize, CellId)>,
    ) -> Vec<Assignment> {
        self.build_candidates(day, state, cost_map, false, excluded, Some(patrol_id))
            .0
    }

    pub(super) fn distribute_agents_with_diagnostics(
        &self,
        day: &DayData,
        state: &PlanningState,
        cost_map: &CostMap,
        excluded: &HashSet<(usize, CellId)>,
    ) -> (Vec<Assignment>, Vec<CandidateCounts>) {
        self.build_candidates(day, state, cost_map, true, excluded, None)
    }

    fn build_candidates(
        &self,
        day: &DayData,
        state: &PlanningState,
        cost_map: &CostMap,
        collect_diagnostics: bool,
        excluded: &HashSet<(usize, CellId)>,
        only_agent: Option<usize>,
    ) -> (Vec<Assignment>, Vec<CandidateCounts>) {
        let mut candidates = Vec::new();
        let mut counts =
            collect_diagnostics.then(|| vec![CandidateCounts::default(); day.agents.len()]);

        for (agent_id, (agent, cursor)) in day.agents.iter().zip(state.cursor.iter()).enumerate() {
            if agent.kind != AgentKind::Patrol || only_agent.is_some_and(|only| only != agent_id) {
                continue;
            }

            let cell_steps = self.dijkstra(cost_map, &cursor.pos);

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

                let steps = cell_steps[spot_id.as_usize()];
                if steps == u32::MAX {
                    if let Some(count) = counts.as_mut().and_then(|counts| counts.get_mut(agent_id))
                    {
                        count.unreachable += 1;
                    }
                    continue;
                }
                if cursor.fixed_steps + steps > day.steps {
                    if let Some(count) = counts.as_mut().and_then(|counts| counts.get_mut(agent_id))
                    {
                        count.exceeds_day += 1;
                    }
                    continue;
                }

                if let Some(count) = counts.as_mut().and_then(|counts| counts.get_mut(agent_id)) {
                    count.feasible += 1;
                    if let Some(spot) = state.spots.get(spot_id) {
                        if state.unvisited_brands.contains(&spot.brand) {
                            count.reachable_unvisited_brands.insert(spot.brand);
                        }
                    }
                }

                candidates.push(SpotCandidate {
                    agent_id,
                    spot_id: *spot_id,
                    steps,
                    slack: day.steps - cursor.fixed_steps - steps,
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
            "distribution day={} agent={} pos={} fixed_steps={} remaining_steps={} fuel={} candidates={{occupied:{}, empty_stock:{}, visited_spot:{}, unreachable:{}, exceeds_day:{}, reachable_within_day:{}, capacity_pruned:{}, retained_after_stock:{}}} reachable_unvisited_brands={:?} retained_spots={:?}",
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
            count.feasible,
            count.capacity_pruned,
            retained_spots.get(agent_id).map_or(0, Vec::len),
            count.reachable_unvisited_brands,
            retained_spots.get(agent_id),
        );
    }
}

/// Limits the number of same spots to the number of stocks by removing the distant spots.
fn limit_candidates_by_stock(state: &PlanningState, candidates: &mut Vec<SpotCandidate>) {
    let mut spot_counts: HashMap<CellId, u32> = HashMap::new();
    let mut index = 0;
    while let Some(spot_id) = candidates.get(index).map(|candidate| candidate.spot_id) {
        let spot_count = match spot_counts.entry(spot_id) {
            Entry::Occupied(mut entry) => {
                let count = entry.get_mut();
                *count += 1;
                *count
            }
            Entry::Vacant(entry) => {
                entry.insert(1);
                1
            }
        };

        let Some(SpotState { stocks, .. }) = state.spots.get(&spot_id) else {
            break;
        };
        if spot_count > *stocks {
            candidates.remove(index);
        } else {
            index += 1;
        }
    }
}

/// Prioritizes scarce unvisited brands and candidates that are close to becoming unreachable.
fn prioritize_candidates(state: &PlanningState, candidates: &mut [SpotCandidate]) {
    const URGENCY_MARGIN: u32 = 5;
    let mut brand_agents = HashMap::new();
    let brand_representatives = find_brand_representatives(state, candidates);

    for candidate in candidates.iter() {
        let Some(spot) = state.spots.get(&candidate.spot_id) else {
            continue;
        };
        if !state.unvisited_brands.contains(&spot.brand) {
            continue;
        }

        brand_agents
            .entry(spot.brand)
            .or_insert_with(HashSet::new)
            .insert(candidate.agent_id);
    }

    candidates.sort_unstable_by_key(|candidate| {
        let Some(spot) = state.spots.get(&candidate.spot_id) else {
            return (true, true, usize::MAX, true, candidate.steps);
        };

        let is_visited = !state.unvisited_brands.contains(&spot.brand);
        let available_agents = brand_agents
            .get(&spot.brand)
            .map_or(usize::MAX, HashSet::len);
        let urgency = !is_visited && candidate.slack > URGENCY_MARGIN;
        let is_representative =
            brand_representatives.contains(&(candidate.agent_id, candidate.spot_id));

        (
            is_visited,
            !is_representative,
            available_agents,
            urgency,
            candidate.steps,
        )
    });
}

fn find_brand_representatives(
    state: &PlanningState,
    candidates: &[SpotCandidate],
) -> HashSet<(usize, CellId)> {
    let mut representatives = HashMap::<_, (usize, CellId, u32)>::new();

    for candidate in candidates {
        let Some(spot) = state.spots.get(&candidate.spot_id) else {
            continue;
        };

        if !state.unvisited_brands.contains(&spot.brand) {
            continue;
        }

        let representative = representatives.entry(spot.brand).or_insert((
            candidate.agent_id,
            candidate.spot_id,
            candidate.steps,
        ));

        if candidate.steps < representative.2 {
            *representative = (candidate.agent_id, candidate.spot_id, candidate.steps);
        }
    }

    representatives
        .into_values()
        .map(|(agent_id, spot_id, _)| (agent_id, spot_id))
        .collect()
}
