use super::Assignment;
use crate::{
    algorithm::Solver,
    game::{Brand, CellId},
};
use std::collections::{HashMap, HashSet};

pub(super) struct SpotCandidate {
    pub(super) spot_id: CellId,
    pub(super) brand: Brand,
    pub(super) distance: u32,
    pub(super) stock: u32,
}

pub(super) struct AgentCandidates {
    pub(super) agent_id: usize,
    pub(super) spots: Vec<SpotCandidate>,
}

impl Solver<'_> {
    pub(super) fn search(&self, agents: &[AgentCandidates]) -> Vec<Assignment> {
        let mut unassigned: HashSet<usize> = agents.iter().map(|agent| agent.agent_id).collect();
        let mut used_stock = HashMap::<CellId, u32>::new();
        let mut collected_brands = HashSet::<Brand>::new();
        let mut assignments = Vec::new();

        while !unassigned.is_empty() {
            let best = agents
                .iter()
                .filter(|agent| unassigned.contains(&agent.agent_id))
                .flat_map(|agent| {
                    agent.spots.iter().filter_map(|candidate| {
                        let used = used_stock.get(&candidate.spot_id).copied().unwrap_or(0);
                        if used >= candidate.stock {
                            return None;
                        }

                        Some((
                            agent.agent_id,
                            candidate,
                            !collected_brands.contains(&candidate.brand),
                        ))
                    })
                })
                .max_by_key(|(agent_id, candidate, adds_brand)| {
                    (
                        *adds_brand,
                        std::cmp::Reverse(candidate.distance),
                        std::cmp::Reverse(candidate.spot_id),
                        std::cmp::Reverse(*agent_id),
                    )
                });

            let Some((agent_id, candidate, adds_brand)) = best else {
                break;
            };

            assignments.push(Assignment {
                agent_id,
                spot: candidate.spot_id,
            });
            *used_stock.entry(candidate.spot_id).or_default() += 1;
            if adds_brand {
                collected_brands.insert(candidate.brand.clone());
            }
            unassigned.remove(&agent_id);
        }

        assignments.sort_unstable_by_key(|assignment| assignment.agent_id);
        assignments
    }
}
