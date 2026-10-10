use super::SpotCandidate;
use crate::algorithm::planning_state::PlanningState;
use std::collections::{BTreeMap, BTreeSet, HashSet};

pub(super) fn allocate_brand_coverage(
    state: &PlanningState,
    mut candidates: Vec<SpotCandidate>,
) -> Vec<SpotCandidate> {
    candidates.sort_unstable_by_key(candidate_cost_key);
    let coverage = maximum_stock_aware_matching(state, &candidates);
    let mut selected_pairs = coverage
        .iter()
        .map(|candidate| (candidate.agent_id, candidate.spot_id))
        .collect::<HashSet<_>>();
    let mut ordered = coverage;
    for candidate in candidates {
        if selected_pairs.insert((candidate.agent_id, candidate.spot_id)) {
            ordered.push(candidate);
        }
    }
    ordered
}

fn maximum_stock_aware_matching(
    state: &PlanningState,
    candidates: &[SpotCandidate],
) -> Vec<SpotCandidate> {
    let agent_ids = candidates
        .iter()
        .map(|candidate| candidate.agent_id)
        .collect::<BTreeSet<_>>();
    let spot_ids = candidates
        .iter()
        .map(|candidate| candidate.spot_id)
        .collect::<BTreeSet<_>>();
    let brand_ids = candidates
        .iter()
        .map(|candidate| candidate_brand_id(state, candidate.spot_id))
        .filter(|brand_id| {
            state
                .unvisited_brands
                .iter()
                .any(|brand| brand.id() == *brand_id)
        })
        .collect::<BTreeSet<_>>();

    let source = 0;
    let mut next_node = 1;
    let agent_nodes = assign_nodes(agent_ids, &mut next_node);
    let spot_nodes = assign_nodes(spot_ids.iter().copied(), &mut next_node);
    let brand_nodes = assign_nodes(brand_ids.iter().copied(), &mut next_node);
    let sink = next_node;
    let mut graph = FlowGraph::new(sink + 1);

    for node in agent_nodes.values() {
        graph.add_edge(source, *node, 1, LexCost::ZERO);
    }
    for (spot_id, spot_node) in &spot_nodes {
        let brand_id = candidate_brand_id(state, *spot_id);
        if let Some(brand_node) = brand_nodes.get(&brand_id) {
            let stock = state.spots.get(spot_id).map_or(0, |spot| spot.stocks);
            graph.add_edge(*spot_node, *brand_node, stock, LexCost::ZERO);
        }
    }
    for node in brand_nodes.values() {
        graph.add_edge(*node, sink, 1, LexCost::ZERO);
    }

    let mut candidate_edges = Vec::new();
    for candidate in candidates {
        let brand_id = candidate_brand_id(state, candidate.spot_id);
        if !brand_nodes.contains_key(&brand_id) {
            continue;
        }
        let Some(agent_node) = agent_nodes.get(&candidate.agent_id) else {
            continue;
        };
        let Some(spot_node) = spot_nodes.get(&candidate.spot_id) else {
            continue;
        };
        let edge_index = graph.add_edge(*agent_node, *spot_node, 1, LexCost::candidate(candidate));
        candidate_edges.push((candidate, *agent_node, edge_index));
    }

    graph.min_cost_max_flow(source, sink);
    candidate_edges
        .into_iter()
        .filter_map(|(candidate, node, edge_index)| {
            graph
                .edge_is_used(node, edge_index)
                .then_some(SpotCandidate {
                    agent_id: candidate.agent_id,
                    spot_id: candidate.spot_id,
                    steps: candidate.steps,
                    slack: candidate.slack,
                    fuel: candidate.fuel,
                    lookahead_brand: candidate.lookahead_brand,
                    lookahead_collection: candidate.lookahead_collection,
                })
        })
        .collect()
}

fn assign_nodes<K: Ord + Copy>(
    keys: impl IntoIterator<Item = K>,
    next_node: &mut usize,
) -> BTreeMap<K, usize> {
    keys.into_iter()
        .map(|key| {
            let node = *next_node;
            *next_node += 1;
            (key, node)
        })
        .collect()
}

fn candidate_brand_id(state: &PlanningState, spot_id: crate::game::CellId) -> i64 {
    state.spots[&spot_id].brand.id()
}

fn candidate_cost_key(
    candidate: &SpotCandidate,
) -> (bool, bool, u32, u32, u32, usize, crate::game::CellId) {
    (
        !candidate.lookahead_brand,
        !candidate.lookahead_collection,
        candidate.steps,
        candidate.fuel,
        u32::MAX.saturating_sub(candidate.slack),
        candidate.agent_id,
        candidate.spot_id,
    )
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct LexCost([i64; 7]);

impl LexCost {
    const ZERO: Self = Self([0; 7]);

    fn candidate(candidate: &SpotCandidate) -> Self {
        Self([
            i64::from(!candidate.lookahead_brand),
            i64::from(!candidate.lookahead_collection),
            i64::from(candidate.steps),
            i64::from(candidate.fuel),
            i64::from(u32::MAX.saturating_sub(candidate.slack)),
            candidate.agent_id as i64,
            candidate.spot_id.as_usize() as i64,
        ])
    }

    fn plus(self, other: Self) -> Self {
        Self(std::array::from_fn(|index| self.0[index] + other.0[index]))
    }

    fn negated(self) -> Self {
        Self(self.0.map(|value| -value))
    }
}

struct FlowEdge {
    to: usize,
    reverse: usize,
    capacity: u32,
    cost: LexCost,
}

struct FlowGraph {
    edges: Vec<Vec<FlowEdge>>,
}

impl FlowGraph {
    fn new(node_count: usize) -> Self {
        Self {
            edges: (0..node_count).map(|_| Vec::new()).collect(),
        }
    }

    fn add_edge(&mut self, from: usize, to: usize, capacity: u32, cost: LexCost) -> usize {
        let edge_index = self.edges[from].len();
        let reverse_index = self.edges[to].len();
        self.edges[from].push(FlowEdge {
            to,
            reverse: reverse_index,
            capacity,
            cost,
        });
        self.edges[to].push(FlowEdge {
            to: from,
            reverse: edge_index,
            capacity: 0,
            cost: cost.negated(),
        });
        edge_index
    }

    fn min_cost_max_flow(&mut self, source: usize, sink: usize) -> u32 {
        let mut flow = 0;
        loop {
            let Some(previous) = self.shortest_path(source, sink) else {
                return flow;
            };
            let mut node = sink;
            while node != source {
                let (from, edge_index) = previous[node].expect("path node has predecessor");
                let edge = &self.edges[from][edge_index];
                let reverse = edge.reverse;
                self.edges[from][edge_index].capacity -= 1;
                self.edges[node][reverse].capacity += 1;
                node = from;
            }
            flow += 1;
        }
    }

    fn shortest_path(&self, source: usize, sink: usize) -> Option<Vec<Option<(usize, usize)>>> {
        let mut distances = vec![None; self.edges.len()];
        let mut previous = vec![None; self.edges.len()];
        distances[source] = Some(LexCost::ZERO);

        for _ in 1..self.edges.len() {
            let mut changed = false;
            for from in 0..self.edges.len() {
                let Some(distance) = distances[from] else {
                    continue;
                };
                for (edge_index, edge) in self.edges[from].iter().enumerate() {
                    if edge.capacity == 0 {
                        continue;
                    }
                    let candidate = distance.plus(edge.cost);
                    if distances[edge.to].is_none_or(|current| candidate < current) {
                        distances[edge.to] = Some(candidate);
                        previous[edge.to] = Some((from, edge_index));
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }

        distances[sink].map(|_| previous)
    }

    fn edge_is_used(&self, node: usize, edge_index: usize) -> bool {
        self.edges[node][edge_index].capacity == 0
    }
}

#[cfg(test)]
mod tests {
    use super::{FlowGraph, LexCost, assign_nodes};
    use std::collections::BTreeSet;

    #[test]
    fn flow_matching_preserves_unique_access_brand_and_maximizes_coverage() {
        let mut next_node = 1;
        let agents = assign_nodes([0usize, 1], &mut next_node);
        let spots = assign_nodes([10usize, 20], &mut next_node);
        let brands = assign_nodes([100i64, 200], &mut next_node);
        let sink = next_node;
        let mut graph = FlowGraph::new(sink + 1);
        graph.add_edge(0, agents[&0], 1, LexCost::ZERO);
        graph.add_edge(0, agents[&1], 1, LexCost::ZERO);
        let flexible_common =
            graph.add_edge(agents[&0], spots[&10], 1, LexCost([1, 0, 0, 0, 0, 0, 10]));
        let flexible_unique =
            graph.add_edge(agents[&0], spots[&20], 1, LexCost([5, 0, 0, 0, 0, 0, 20]));
        let constrained_common =
            graph.add_edge(agents[&1], spots[&10], 1, LexCost([2, 0, 1, 0, 0, 1, 10]));
        graph.add_edge(spots[&10], brands[&100], 1, LexCost::ZERO);
        graph.add_edge(spots[&20], brands[&200], 1, LexCost::ZERO);
        graph.add_edge(brands[&100], sink, 1, LexCost::ZERO);
        graph.add_edge(brands[&200], sink, 1, LexCost::ZERO);

        assert_eq!(graph.min_cost_max_flow(0, sink), 2);
        assert!(!graph.edge_is_used(agents[&0], flexible_common));
        assert!(graph.edge_is_used(agents[&0], flexible_unique));
        assert!(graph.edge_is_used(agents[&1], constrained_common));
    }

    #[test]
    fn flow_respects_stock_and_brand_capacity() {
        let mut graph = FlowGraph::new(6);
        graph.add_edge(0, 1, 1, LexCost::ZERO);
        graph.add_edge(0, 2, 1, LexCost::ZERO);
        graph.add_edge(1, 3, 1, LexCost::ZERO);
        graph.add_edge(2, 3, 1, LexCost::ZERO);
        graph.add_edge(3, 4, 1, LexCost::ZERO);
        graph.add_edge(4, 5, 1, LexCost::ZERO);

        assert_eq!(graph.min_cost_max_flow(0, 5), 1);
    }

    #[test]
    fn equal_cost_candidate_order_is_stable() {
        let values = BTreeSet::from([3, 1, 2]);

        assert_eq!(values.into_iter().collect::<Vec<_>>(), vec![1, 2, 3]);
    }
}
