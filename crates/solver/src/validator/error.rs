/// Describes one invalid part of a generated action plan.
#[derive(Debug)]
pub struct PlanValidationError {
    pub agent_id: usize,
    pub message: String,
}

impl PlanValidationError {
    pub(super) fn new(agent_id: usize, message: impl Into<String>) -> Self {
        Self {
            agent_id,
            message: message.into(),
        }
    }
}
