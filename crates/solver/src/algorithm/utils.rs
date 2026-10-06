use crate::{
    algorithm::{AgentCursor, Solver},
    game::{Action, CellId, CellType, DayData},
};

impl Solver<'_> {
    /// Calculates the number of steps it takes for the given action.
    pub(super) fn get_action_steps(
        &self,
        day: &DayData,
        cursor: &AgentCursor,
        action: &Action,
    ) -> u32 {
        match action {
            Action::Move(_) => self.get_cell_steps(day, &cursor.pos),
            Action::Wait(steps) => steps.get(),
        }
    }

    /// Gets the number of steps it takes to move through the cell.
    pub(super) fn get_cell_steps(&self, day: &DayData, id: &CellId) -> u32 {
        match self.map.get_cell(id) {
            Some(cell) => match cell {
                CellType::Plain => 2,
                CellType::Mountain => 3,
                CellType::Road => {
                    if let Some(traffic) = day.traffics.get(id) {
                        traffic.steps()
                    } else {
                        u32::MAX
                    }
                }
                CellType::Pond => u32::MAX,
            },
            None => u32::MAX,
        }
    }
}
