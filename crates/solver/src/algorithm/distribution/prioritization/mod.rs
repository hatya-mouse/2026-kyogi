use super::SpotCandidate;
use crate::algorithm::{PlanningState, planning_state::SpotState};
use crate::game::{Brand, CellId};
use std::collections::{HashMap, HashSet, hash_map::Entry};

/// Limits candidates at a spot to its remaining collection capacity.
pub(super) fn limit_candidates_by_stock(
    state: &PlanningState,
    candidates: &mut Vec<SpotCandidate>,
) {
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
pub(super) fn prioritize_candidates(state: &PlanningState, candidates: &mut [SpotCandidate]) {
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
            return (
                true,
                true,
                usize::MAX,
                true,
                true,
                true,
                candidate.steps,
                candidate.fuel,
                candidate.agent_id,
                candidate.spot_id,
            );
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
            !candidate.lookahead_brand,
            !candidate.lookahead_collection,
            candidate.steps,
            candidate.fuel,
            candidate.agent_id,
            candidate.spot_id,
        )
    });
}

fn find_brand_representatives(
    state: &PlanningState,
    candidates: &[SpotCandidate],
) -> HashSet<(usize, CellId)> {
    let mut representatives = HashMap::<Brand, (usize, CellId, u32, u32)>::new();

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
            candidate.fuel,
        ));
        let candidate_key = (
            candidate.steps,
            candidate.fuel,
            candidate.agent_id,
            candidate.spot_id,
        );
        let representative_key = (
            representative.2,
            representative.3,
            representative.0,
            representative.1,
        );

        if candidate_key < representative_key {
            *representative = (
                candidate.agent_id,
                candidate.spot_id,
                candidate.steps,
                candidate.fuel,
            );
        }
    }

    representatives
        .into_values()
        .map(|(agent_id, spot_id, _, _)| (agent_id, spot_id))
        .collect()
}
