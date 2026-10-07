use crate::game::{CellId, CellType, DayData, Map};

/// Returns the number of steps required to leave a cell.
pub(super) fn move_steps(map: &Map, day: &DayData, position: CellId) -> u32 {
    match map.get_cell(&position) {
        Some(CellType::Plain) => 2,
        Some(CellType::Mountain) => 3,
        Some(CellType::Road) => day
            .traffics
            .get(&position)
            .map_or(u32::MAX, |traffic| traffic.steps()),
        Some(CellType::Pond) | None => u32::MAX,
    }
}

/// Returns the fuel required to leave a cell.
pub(super) fn move_fuel(map: &Map, position: CellId) -> u32 {
    match map.get_cell(&position) {
        Some(CellType::Plain) => 1,
        Some(CellType::Mountain | CellType::Road) => 2,
        Some(CellType::Pond) | None => u32::MAX,
    }
}

/// Returns the position reached by a movement action.
pub(super) fn next_position(
    map: &Map,
    position: CellId,
    direction: crate::game::Direction,
) -> Option<CellId> {
    let coord = map.get_coord_from_id(position);
    let next = map.get_id_from_coord(direction.apply_to_coord(coord));

    map.get_neighbors(position)
        .into_iter()
        .find(|candidate| *candidate == next)
}
