use crate::game::{CellId, CellType, Map};
use std::collections::HashMap;

pub(super) struct AdjGraph {
    edges: HashMap<CellId, Vec<CellId>>,
}

impl AdjGraph {
    /// Builds a new adjacent graph from the given map.
    pub(super) fn build(map: &Map) -> Self {
        let cell_count = map.cell_count();
        let mut edges = HashMap::with_capacity(cell_count);

        for cell_idx in 0..cell_count {
            // If the cell is Pond, skip it
            let cell_id = CellId(cell_idx);
            if matches!(map.get_cell(&cell_id), Some(CellType::Pond)) {
                continue;
            }

            // Collect the neighbors of the cell and add it
            edges.insert(cell_id, map.get_neighbors(cell_id));
        }

        Self { edges }
    }

    /// Returns the neighbors of the given cell.
    /// This function returns None if the given cell does not exist, or the given cell is Pond.
    pub(super) fn neighbors(&self, cell_id: &CellId) -> Option<&Vec<CellId>> {
        self.edges.get(cell_id)
    }
}
