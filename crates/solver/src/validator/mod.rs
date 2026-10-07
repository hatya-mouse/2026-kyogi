mod error;
mod movement;
mod trace;

use crate::game::{AgentKind, DayData, DayPlan, Map};
use trace::{AgentTrace, build_trace};

pub use error::PlanValidationError;

/// Validates a generated plan against the server movement rules.
pub fn validate_day_plan(
    map: &Map,
    day: &DayData,
    plan: &DayPlan,
    fuel_limit: u32,
) -> Vec<PlanValidationError> {
    if plan.actions.len() != day.agents.len() {
        return vec![PlanValidationError::new(
            usize::MAX,
            format!(
                "expected {} agents but plan contains {}",
                day.agents.len(),
                plan.actions.len()
            ),
        )];
    }

    let mut traces = Vec::with_capacity(day.agents.len());
    let mut errors = Vec::new();

    for (agent_id, (agent, actions)) in day.agents.iter().zip(&plan.actions).enumerate() {
        match build_trace(map, day, agent_id, agent, actions) {
            Ok(trace) => traces.push(trace),
            Err(error) => errors.push(error),
        }
    }

    validate_fuel(&traces, fuel_limit, &mut errors);
    errors
}

fn validate_fuel(traces: &[AgentTrace], fuel_limit: u32, errors: &mut Vec<PlanValidationError>) {
    for (agent_id, trace) in traces.iter().enumerate() {
        if trace.kind != AgentKind::Patrol {
            continue;
        }

        let mut fuel = trace.initial_fuel;
        for movement in &trace.moves {
            if movement.fuel > fuel {
                errors.push(PlanValidationError::new(
                    agent_id,
                    format!(
                        "movement at step {} exceeds available fuel",
                        movement.arrival_step
                    ),
                ));
                continue;
            }

            fuel -= movement.fuel;
            if has_supply_at_step(
                traces,
                agent_id,
                movement.arrival_step,
                trace.positions[movement.arrival_step as usize],
            ) {
                fuel = fuel_limit;
            }
        }
    }
}

fn has_supply_at_step(
    traces: &[AgentTrace],
    patrol_id: usize,
    step: u32,
    position: crate::game::CellId,
) -> bool {
    traces.iter().enumerate().any(|(agent_id, trace)| {
        agent_id != patrol_id
            && trace.kind == AgentKind::Supply
            && trace.positions.get(step as usize) == Some(&position)
    })
}
