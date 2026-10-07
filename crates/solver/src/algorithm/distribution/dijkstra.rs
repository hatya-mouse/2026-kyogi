use crate::{
    algorithm::{CostMap, Solver},
    game::CellId,
};
use std::{cmp::Reverse, collections::BinaryHeap};

impl Solver<'_> {
    /// Calculates the number of steps it takes to move to the spots from the src cell.
    pub(super) fn dijkstra_backward(&self, cost_map: &CostMap, src: &CellId) -> Vec<u32> {
        // Create a priority queue that stores (distance, cell)
        let mut steps = vec![u32::MAX; self.board.map.cell_count()];
        let mut pq = BinaryHeap::new();

        // Steps to the source cell is zero
        pq.push(Reverse((0, src)));
        steps[src.as_usize()] = 0;

        while !pq.is_empty() {
            if let Some(Reverse((current_steps, current))) = pq.pop() {
                let current_idx = current.as_usize();

                if current_steps > steps[current_idx] {
                    continue;
                }

                let Some(move_cost) = cost_map.steps(current) else {
                    continue;
                };

                // Loop neighbors and update the number of steps to the adjacent spots
                let Some(neighbors) = self.adj_graph.neighbors(current) else {
                    continue;
                };
                for neighbor in neighbors {
                    let neighbor_idx = neighbor.as_usize();

                    // Get the number of steps to move TO current cell FROM the neighbor cell
                    let new_cost = current_steps.saturating_add(move_cost);

                    let Some(old_cost) = steps.get(neighbor_idx) else {
                        continue;
                    };
                    if new_cost < *old_cost {
                        // Update the cost if the new cost is smaller
                        steps[neighbor_idx] = new_cost;
                        pq.push(Reverse((new_cost, neighbor)));
                    }
                }
            }
        }

        steps
    }
}
