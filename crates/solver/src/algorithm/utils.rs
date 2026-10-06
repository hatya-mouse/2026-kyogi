use crate::{
    algorithm::AgentCursor,
    game::{Action, CellId, CellType, DayData, Map},
};

/// Calculates the number of steps it takes for the given action.
pub(super) fn get_action_steps(
    map: &Map,
    day: &DayData,
    cursor: &AgentCursor,
    action: &Action,
) -> u32 {
    match action {
        Action::Move(_) => get_cell_steps(map, day, &cursor.pos),
        Action::Wait(steps) => steps.get(),
    }
}

/// Gets the number of steps it takes to move through the cell.
pub(super) fn get_cell_steps(map: &Map, day: &DayData, id: &CellId) -> u32 {
    match map.get_cell(id) {
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
