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
        let mut checked_until = 0;

        for movement in &trace.moves {
            // A supply car may meet a patrol car while it is waiting or moving
            if has_supply_between_steps(
                traces,
                agent_id,
                trace,
                checked_until,
                movement.departure_step,
            ) {
                fuel = fuel_limit;
            }

            if movement.fuel > fuel {
                errors.push(PlanValidationError::new(
                    agent_id,
                    format!(
                        "movement at step {} exceeds available fuel",
                        movement.arrival_step
                    ),
                ));
            } else {
                fuel -= movement.fuel;
            }

            // The movement itself consumes fuel before its arrival can refill it
            if has_supply_between_steps(
                traces,
                agent_id,
                trace,
                movement.departure_step.saturating_add(1),
                movement.arrival_step,
            ) {
                fuel = fuel_limit;
            }

            checked_until = movement.arrival_step.saturating_add(1);
        }
    }
}

fn has_supply_between_steps(
    traces: &[AgentTrace],
    patrol_id: usize,
    patrol: &AgentTrace,
    first_step: u32,
    last_step: u32,
) -> bool {
    (first_step..=last_step).any(|step| {
        let Some(&position) = patrol.positions.get(step as usize) else {
            return false;
        };

        traces.iter().enumerate().any(|(agent_id, trace)| {
            agent_id != patrol_id
                && trace.kind == AgentKind::Supply
                && trace.positions.get(step as usize) == Some(&position)
        })
    })
}
