use crate::game::CellId;
use serde::Deserialize;
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::fmt::Display;

#[derive(Deserialize_repr, Serialize_repr, Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum AgentKind {
    Patrol = 0,
    Supply = 1,
}

impl Display for AgentKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentKind::Patrol => write!(f, "Patrol"),
            AgentKind::Supply => write!(f, "Supply"),
        }
    }
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
