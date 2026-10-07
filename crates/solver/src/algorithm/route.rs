use crate::{
    algorithm::{CostMap, Solver},
    game::{Action, CellId},
};
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap, HashSet},
};

impl Solver<'_> {
    /// Converts a route represented by cells into movement actions.
    pub(super) fn route_to_actions(&self, route: &[CellId]) -> Option<Vec<Action>> {
        route
            .windows(2)
            .map(|pair| {
                self.board
                    .map
                    .direction_to(pair[0], pair[1])
                    .map(Action::Move)
            })
            .collect()
    }

    pub(super) fn get_route(&self, cost_map: &CostMap, src: CellId, dst: CellId) -> Vec<CellId> {
        // Initialize a step array with zeros
        let mut steps = vec![u32::MAX; self.board.map.cell_count()];
        let mut closed = HashSet::new();
        let mut open = BinaryHeap::new();
        let mut previous = HashMap::new();

        // Push the src cell to the open list
        open.push(Reverse((0, src)));

        // Set the cost of the src cell to zero
        steps[src.as_usize()] = 0;

        while !open.is_empty() {
            // Calculate the cost for each candidate and take the one with the lowest cost
            if let Some(Reverse((_, current))) = open.pop() {
                // Add the current cell to the visited list
                closed.insert(current);

                // If the current cell is the dst cell, exit the loop
                if current == dst {
                    break;
                }

                let Some(move_cost) = cost_map.steps(&current) else {
                    continue;
                };

                // Get the neighbors of the current cell
                let Some(neighbors) = self.adj_graph.neighbors(&current) else {
                    continue;
                };

                for neighbor in neighbors {
                    // If the neighbor has already been visited, skip it
                    if closed.contains(neighbor) {
                        continue;
                    }

                    // Calculate the actual cost from the src to the neighbor
                    let actual_cost = steps[current.as_usize()].saturating_add(move_cost);

                    // If the actual cost to the neighbor is less than the current cost,
                    // update the cost and add the neighbor to the candidates list
                    if actual_cost < steps[neighbor.as_usize()] {
                        steps[neighbor.as_usize()] = actual_cost;
                        previous.insert(neighbor, current);

                        let est_total =
                            actual_cost.saturating_add(self.board.map.distance(*neighbor, dst));
                        open.push(Reverse((est_total, *neighbor)));
                    }
                }
            }
        }

        // Backtrack from the dst to the src to get the route
        let mut current = dst;
        let mut route = vec![current];
        while current != src {
            if let Some(&prev) = previous.get(&current) {
                current = prev;
                route.push(current);
            } else {
                // If there is no route from the dst to the src, return an empty vector
                return Vec::new();
            }
        }
        route.reverse();
        route
    }
}
