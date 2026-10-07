use serde::Serialize;
use serde_repr::Serialize_repr;
use std::num::NonZeroU32;

// --- DayPlan ---

/// A plan for a single day.
#[derive(Clone, Debug)]
pub struct DayPlan {
    /// Actions for each agent for the day.
    pub actions: Vec<Vec<Action>>,
}

impl DayPlan {
    pub fn new(agents_num: usize) -> Self {
        Self {
            actions: vec![Vec::new(); agents_num],
        }
    }

    /// Adds an action to the specific agent in the plan.
    pub fn add_action(&mut self, agent_id: usize, action: Action) {
        if let Some(agent_actions) = self.actions.get_mut(agent_id) {
            agent_actions.push(action);
        }
    }

    /// Adds multiple actions to the specific agent in the plan.
    pub fn extend_actions(&mut self, agent_id: usize, actions: Vec<Action>) {
        if let Some(agent_actions) = self.actions.get_mut(agent_id) {
            agent_actions.extend(actions);
        }
    }
}

// --- Direction ---

/// A diretion on the map.
#[derive(Serialize_repr, Debug, Clone, Copy)]
#[repr(u8)]
pub enum Direction {
    TopLeft = 0,
    TopRight = 1,
    Right = 2,
    BottomRight = 3,
    BottomLeft = 4,
    Left = 5,
}

impl Direction {
    /// Calculates the position of the agent after moving to the direction.
    ///
    /// # Arguments
    /// - `coord`: The current position of the agent.
    pub fn apply_to_coord(&self, coord: (usize, usize)) -> (usize, usize) {
        let is_odd = coord.1 & 1;
        let is_even = is_odd ^ 1;

        match self {
            Direction::TopLeft => (coord.0.saturating_sub(is_odd), coord.1.saturating_sub(1)),
            Direction::TopRight => (coord.0.saturating_add(is_even), coord.1.saturating_sub(1)),
            Direction::Right => (coord.0.saturating_add(1), coord.1),
            Direction::BottomRight => (coord.0.saturating_add(is_even), coord.1.saturating_add(1)),
            Direction::BottomLeft => (coord.0.saturating_sub(is_odd), coord.1.saturating_add(1)),
            Direction::Left => (coord.0.saturating_sub(1), coord.1),
        }
    }
}

// --- Action ---

/// An enum that represents a single action in the action plan.
#[derive(Debug, Clone, Copy)]
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
