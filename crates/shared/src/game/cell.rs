use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::fmt::Debug;

// --- CellType ---

/// Cell data that stores the type of the cell.
#[derive(Deserialize_repr, Serialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum CellType {
    Plain = 0,
    Road = 1,
    Mountain = 2,
    Pond = 3,
}

// --- TrafficStatus ---

/// The traffic status of the road tile.
#[derive(Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum TrafficStatus {
    Smooth = 0,
    Busy = 1,
    Jammed = 2,
}

// --- CellId ---

/// An ID that represents the position on the map.
#[derive(Serialize, Deserialize)]
pub struct CellId(pub usize);

impl Debug for CellId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl CellId {
    #[inline]
    pub fn x(self, width: usize) -> usize {
        self.0 % width
    }

    #[inline]
    pub fn y(self, width: usize) -> usize {
        self.0 / width
    }

    #[inline]
    pub fn to_coord(self, width: usize) -> (usize, usize) {
        (self.0 % width, self.0 / width)
    }
}
