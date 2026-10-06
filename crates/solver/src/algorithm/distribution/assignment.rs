use crate::{algorithm::Solver, game::CellId};

impl Solver<'_> {
    pub(super) fn search(
        &self,
        agents_to_spots: &[(usize, Vec<(CellId, u32)>)],
    ) -> Vec<(usize, CellId)> {
        let mut result = Vec::new();

        for (agent_id, spots_dist) in agents_to_spots {
            if let Some(nearest) = spots_dist.first() {
                result.push((*agent_id, nearest.0));
            }
        }

        result
    }
}
