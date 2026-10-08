use crate::game::{Brand, CellId, Map, Spot};
use std::collections::HashMap;

/// Board state that never changes throughout the game.
pub(crate) struct Board<'a> {
    /// The current map of the game.
    pub map: &'a Map,
    /// Spots on the map.
    pub spots: HashMap<CellId, Spot>,
    /// Brands and their corresponding spot ids in the map.
    pub brands: HashMap<Brand, Vec<CellId>>,
    /// Count of agents on the map.
    pub agent_count: usize,
}

impl<'a> Board<'a> {
    pub(crate) fn new(map: &'a Map, spots: HashMap<CellId, Spot>, agent_count: usize) -> Self {
        // Collect the brands and spots with the brand
        let brands: HashMap<Brand, Vec<CellId>> = spots
            .values()
            .map(|spot| {
                let brand = *spot.brand();
                (
                    brand,
                    spots
                        .iter()
                        .filter_map(|(id, spot)| {
                            if *spot.brand() == brand {
                                Some(*id)
                            } else {
                                None
                            }
                        })
                        .collect(),
                )
            })
            .collect();

        Self {
            map,
            spots,
            brands,
            agent_count,
        }
    }
}
