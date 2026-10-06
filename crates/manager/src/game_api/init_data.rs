use serde::Deserialize;
use shared::game::{CellId, CellType};

/// Initial data that can be deserialized from the JSON data given by the game server.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ApiInitialData {
    /// UNIX time when the game starts.
    pub starts_at: i64,
    /// Response time limit for each day.
    pub day_seconds: Vec<i64>,
    /// Number of steps for each day.
    pub day_steps: Vec<i64>,
    /// The map.
    pub map: ApiMap,
    /// The spots on the map.
    pub spots: Vec<ApiSpot>,
    /// Initial position of the agents.
    pub agents: Vec<CellId>,
    /// The maximum amount of fuel.
    pub fuel_limits: i64,
    /// Number of players.
    pub players: i64,
    /// A threshold where the road turns to "busy" state.
    pub busy_threshold: i64,
    /// A threshold where the road turns to "jammed" state.
    pub jammed_threshold: i64,
}

/// Map data that can be deserialized from the JSON data given by the game server.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ApiMap {
    /// The height of the map.
    pub height: i64,
    /// The width of the map.
    pub width: i64,
    /// Cells of the map.
    pub cells: Vec<Vec<CellType>>,
}

/// JSON-compatible spot data that stores its brand ID, position and number of stocks.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ApiSpot {
    /// The brand ID of the spot.
    pub brand: i64,
    /// The position of the spot.
    pub pos: CellId,
    /// Number of stocks of the spot.
    pub stocks: i64,
}
