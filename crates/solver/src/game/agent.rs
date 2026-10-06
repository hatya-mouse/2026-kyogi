use crate::game::CellId;
use serde::Deserialize;
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Deserialize_repr, Serialize_repr, Debug, PartialEq)]
#[repr(u8)]
pub enum AgentKind {
    Patrol = 0,
    Supply = 1,
}

/// The data of the agent.
#[derive(Deserialize, Debug)]
pub struct Agent {
    /// Kind of the agent.
    pub kind: AgentKind,
    /// Current position of the agent.
    pub pos: CellId,
    /// AMount of the remaining fuel.
    pub fuel: u32,
}
