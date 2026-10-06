use serde::Deserialize;
use solver::game::{Agent, CellId, TrafficStatus};

/// The data for the day that can be deserialized from the JSON data provided by the game server.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ApiDayData {
    /// UNIX time when the day will end.
    pub ends_at: u64,
    /// The current index of the day.
    pub day: u32,
    /// States of the agents at the start of the day.
    pub agents: Vec<Agent>,
    /// States of the opponents' agents at the start of the day.
    pub others: Vec<ApiOtherAgentsData>,
    /// Current traffic status of the roads on the map.
    pub traffics: Vec<ApiTrafficData>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct ApiOtherAgentsData {
    /// ID of the opponent.
    pub id: i64,
    /// States of the opponent's agents.
    pub agents: Vec<Agent>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct ApiTrafficData {
    /// The position of the road tile that this data points to.
    pub pos: CellId,
    /// The traffic status of the road tile.
    pub status: TrafficStatus,
}
