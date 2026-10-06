use crate::{
    algorithm::Solver,
    game::{CellId, DayData},
};
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap, HashSet},
};

impl Solver<'_> {
    pub(crate) fn get_route(&self, day: &DayData, start: CellId, end: CellId) -> Vec<CellId> {
        // Initialize a cost array with zeros
        let mut costs = vec![u32::MAX; self.map.width() * self.map.height()];
        let mut closed = HashSet::new();
        let mut open = BinaryHeap::new();
        let mut previous = HashMap::new();

        // Push the start node to the open list
        open.push(Reverse((CellId::ZERO, start)));

        // Set the cost of the start node to zero
        costs[start.as_usize()] = 0;

        while !open.is_empty() {
            // Calculate the cost for each candidate and take the one with the lowest cost
            if let Some(Reverse((_, current))) = open.pop() {
                // Add the current node to the visited list
                closed.insert(current);

                // If the current node is the end node, exit the loop
                if current == end {
                    break;
                }

                let move_cost = self.get_cell_steps(day, &current);

                // Get the neighbors of the current node
                for neighbor in self.map.get_neighbors(current) {
                    // If the neighbor has already been visited, skip it
                    if closed.contains(&neighbor) {
                        continue;
                    }

                    // Calculate the actual cost from the start to the neighbor
                    let actual_cost = costs[current.as_usize()].saturating_add(move_cost);

                    // If the actual cost to the neighbor is less than the current cost,
                    // update the cost and add the neighbor to the candidates list
                    if actual_cost < costs[neighbor.as_usize()] {
                        costs[neighbor.as_usize()] = actual_cost;
                        previous.insert(neighbor, current);

                        let est_total =
                            actual_cost.saturating_add(self.map.distance(neighbor, end));
                        open.push(Reverse((CellId(est_total as usize), neighbor)));
                    }
                }
            }
        }

        // Backtrack from the end to the start to get the route
        let mut current = end;
        let mut route = vec![current];
        while current != start {
            if let Some(&prev) = previous.get(&current) {
                current = prev;
                route.push(current);
            } else {
                // If there is no route from the end to the start, return an empty vector
                return Vec::new();
            }
        }
        route.reverse();
        route
    }
}
