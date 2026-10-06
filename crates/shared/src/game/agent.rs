use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Deserialize_repr, Serialize_repr, Debug)]
#[repr(u8)]
pub enum AgentKind {
    Patrol = 0,
    Supply = 1,
}
