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

impl CellType {
    pub fn is_passable(&self) -> bool {
        !matches!(self, Self::Pond)
    }
}

// --- TrafficStatus ---

/// The traffic status of the road tile.
#[derive(Deserialize_repr, PartialEq, Debug, Default)]
#[repr(u8)]
pub enum TrafficStatus {
    #[default]
    Smooth = 0,
    Busy = 1,
    Jammed = 2,
}

impl TrafficStatus {
    /// Returns the number of steps it takes to pass this road cell.
    pub fn steps(&self) -> u32 {
        match self {
            TrafficStatus::Smooth => 1,
            TrafficStatus::Busy => 2,
            TrafficStatus::Jammed => 4,
        }
    }
}

// --- CellId ---

/// An ID that represents the position on the map.
#[derive(Serialize, Deserialize, Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct CellId(pub usize);

impl Debug for CellId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl CellId {
    pub const ZERO: CellId = CellId(0);

    #[inline]
    pub fn as_usize(&self) -> usize {
        self.0
    }

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

    #[inline]
    pub fn from_coord(coord: (usize, usize), width: usize) -> Self {
        Self(coord.1 * width + coord.0)
    }
}
