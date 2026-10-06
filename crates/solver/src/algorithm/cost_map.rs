use crate::{
    algorithm::utils::get_cell_steps,
    game::{CellId, DayData, Map},
};

/// Move costs of each cells for the current day.
pub(super) struct CostMap {
    steps: Vec<u32>,
}

impl CostMap {
    pub(super) fn build(map: &Map, day: &DayData) -> Self {
        let cell_count = map.cell_count();
        let mut steps = Vec::with_capacity(cell_count);

        // Loop through cells and get the steps it takes to go outside of the cell
        for cell_idx in 0..cell_count {
            let step = get_cell_steps(map, day, &CellId(cell_idx));
            steps.push(step);
        }

        Self { steps }
    }

    pub(super) fn steps(&self, id: &CellId) -> Option<u32> {
        self.steps.get(id.as_usize()).copied()
    }
}
