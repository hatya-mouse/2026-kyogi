use crate::{
    game::{Action, Agent, AgentKind, CellId, DayData, Map},
    validator::{
        error::PlanValidationError,
        movement::{move_fuel, move_steps, next_position},
    },
};

pub(super) struct AgentTrace {
    pub kind: AgentKind,
    pub initial_fuel: u32,
    pub positions: Vec<CellId>,
    pub moves: Vec<MoveTrace>,
}

pub(super) struct MoveTrace {
    pub arrival_step: u32,
    pub fuel: u32,
}

pub(super) fn build_trace(
    map: &Map,
    day: &DayData,
    agent_id: usize,
    agent: &Agent,
    actions: &[Action],
) -> Result<AgentTrace, PlanValidationError> {
    let mut position = agent.pos;
    let mut elapsed_steps: u32 = 0;
    let mut positions = vec![position];
    let mut moves = Vec::new();

    for action in actions {
        match action {
            Action::Wait(wait) => {
                elapsed_steps = elapsed_steps.saturating_add(wait.get());
                append_positions(&mut positions, position, wait.get());
            }
            Action::Move(direction) => {
                let steps = move_steps(map, day, position);
                let Some(next) = next_position(map, position, *direction) else {
                    return Err(PlanValidationError::new(
                        agent_id,
                        "movement direction is not legal",
                    ));
                };

                elapsed_steps = elapsed_steps.saturating_add(steps);
                append_positions(&mut positions, position, steps.saturating_sub(1));
                positions.push(next);
                moves.push(MoveTrace {
                    arrival_step: elapsed_steps,
                    fuel: move_fuel(map, position),
                });
                position = next;
            }
        }

        if elapsed_steps > day.steps {
            return Err(PlanValidationError::new(
                agent_id,
                "plan exceeds the daily step limit",
            ));
        }
    }

    if elapsed_steps != day.steps {
        return Err(PlanValidationError::new(
            agent_id,
            format!("plan uses {} of {} steps", elapsed_steps, day.steps),
        ));
    }

    Ok(AgentTrace {
        kind: agent.kind.clone(),
        initial_fuel: agent.fuel,
        positions,
        moves,
    })
}

fn append_positions(positions: &mut Vec<CellId>, position: CellId, steps: u32) {
    positions.extend(std::iter::repeat_n(position, steps as usize));
}
