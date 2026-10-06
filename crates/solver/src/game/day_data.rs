use crate::game::{Agent, CellId, TrafficStatus};
use std::collections::HashMap;

pub struct DayData {
    /// The current index of the day.
    pub day: i32,
    /// Number of steps in the day.
    pub steps: u32,
    /// States of the agents at the start of the day.
    pub agents: Vec<Agent>,
    /// Current traffic status of the roads on the map.
    pub traffics: HashMap<CellId, TrafficStatus>,
}
