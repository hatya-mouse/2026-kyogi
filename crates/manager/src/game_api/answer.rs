use serde::Serialize;
use shared::game::{Action, AgentKind};

/// A struct for answering agent kind that can be serialized into a compatible JSON.
#[derive(Serialize, Debug)]
pub(crate) struct ApiAgentKindAnswer(pub Vec<AgentKind>);

/// A struct for answering the action plan that can be serialized into a compatible JSON.
/// The count of the vector must match the number of agents.
#[derive(Serialize, Debug)]
pub(crate) struct ApiActionPlanAnswer(pub Vec<Vec<Action>>);

/// The response returned by the game server when submitting an action plan.
#[derive(serde::Deserialize)]
pub(crate) struct PostPlanResponse {
    revision: i64,
}
