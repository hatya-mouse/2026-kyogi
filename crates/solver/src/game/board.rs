use crate::game::{Brand, CellId, Map, Spot};
use std::collections::{HashMap, HashSet};

/// Board state that never changes throughout the game.
pub(crate) struct Board<'a> {
    /// The current map of the game.
    pub map: &'a Map,
    /// Spots on the map.
    pub spots: HashMap<CellId, Spot>,
    /// Brands in the map.
    pub brands: HashSet<Brand>,
}

impl<'a> Board<'a> {
    pub(crate) fn new(map: &'a Map, spots: HashMap<CellId, Spot>) -> Self {
        let brands: HashSet<Brand> = spots.iter().map(|(_, spot)| *spot.brand()).collect();
        Self { map, spots, brands }
    }
}
