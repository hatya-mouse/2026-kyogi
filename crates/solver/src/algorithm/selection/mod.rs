mod dijkstra;

use crate::{
    algorithm::{AgentCursor, PlanningState, Solver},
    game::{CellId, DayData},
};

impl Solver<'_> {
    pub(super) fn select_spot(
        &self,
        day: &DayData,
        state: &PlanningState,
        cursor: &AgentCursor,
    ) -> Option<CellId> {
    }
}
