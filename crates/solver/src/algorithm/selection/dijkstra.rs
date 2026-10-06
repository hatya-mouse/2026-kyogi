use crate::{
    algorithm::Solver,
    game::{CellId, DayData},
};
use std::{cmp::Reverse, collections::BinaryHeap};

impl Solver<'_> {
    /// Calculates the number of steps to all cells from the given cell.
    fn dijkstra(&self, day: &DayData, src: &CellId) -> Vec<u32> {
        // Create a priority queue that stores (distance, cell)
        let mut steps = vec![u32::MAX; self.map.cell_count()];
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

                // Get the number of steps at the current cell
                let move_cost = self.get_cell_steps(day, current);

                // Loop neighbors and update the number of steps to the adjacent spots
                let Some(neighbors) = self.adj_graph.neighbors(current) else {
                    continue;
                };
                for neighbor in neighbors {
                    let neighbor_idx = neighbor.as_usize();
                    let new_cost = steps[neighbor_idx].saturating_add(move_cost);

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
