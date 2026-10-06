use serde::{Deserialize, Serialize};

/// An identifier used to distinguish workers.
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct WorkerId(pub u8);
