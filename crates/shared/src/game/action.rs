use serde::Serialize;
use serde_repr::Serialize_repr;
use std::num::NonZeroU32;

/// A diretion on the map.
#[derive(Serialize_repr, Debug)]
#[repr(u8)]
pub enum Direction {
    TopLeft = 0,
    TopRight = 1,
    Right = 2,
    BottomRight = 3,
    BottomLeft = 4,
    Left = 5,
}

/// An enum that represents a single action in the action plan.
#[derive(Debug)]
pub enum Action {
    Move(Direction),
    Wait(NonZeroU32),
}

impl Serialize for Action {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Action::Move(dir) => dir.serialize(serializer),
            Action::Wait(steps) => serializer.serialize_i32(-(steps.get() as i32)),
        }
    }
}
