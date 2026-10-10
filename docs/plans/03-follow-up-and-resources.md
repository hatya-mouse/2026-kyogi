# Phase 3: Improve follow-up work and resource awareness

## Goal

Once first-stop brand coverage is sound, improve additional collections and reduce wasted assignment attempts by evaluating remaining time and patrol fuel more directly. Keep the scope on assignment decisions; do not replace A*.

## Resource-feasible candidate evaluation

- Estimate movement fuel by summing fuel costs of departure cells along each route, using the same terrain semantics as the planner.
- Exclude a candidate only when it is genuinely infeasible under current fuel and known refill timing. Do not assume a future refill unless the supply plan can support it.
- Use actual arrival time: `cursor.fixed_steps + route_steps`. Calculate remaining slack from that arrival, not route steps alone.
- Respect movement completion, fuel consumption, and refill timing. A refill processed after a move cannot rescue fuel needed for that same completed move. Keep `PlanningState::try_add_actions` and route validation as final feasibility authorities.
- If exact future refill feasibility is unavailable before supply planning, use a conservative estimate and retain the existing execution-time fallback.

## Short-horizon follow-up selection

- After distinct-brand coverage, consider a small bounded sequence of additional spots per patrol, such as one or two next stops.
- Score additional work by marginal official value: a new daily brand outranks another ball of a brand already collected; additional balls matter after brand objectives.
- Include travel steps, fuel, remaining day, stock, spots already visited by the patrol, and whether a required refill can be scheduled.
- Start with greedy marginal insertion or bounded two-stop lookahead. Avoid full-day combinatorial route optimization unless benchmarks show the simpler approach is inadequate.
- Recompute marginal values after each accepted target because stocks and unvisited brands change.

## Supply interaction follow-up

Only after patrol assignment improvements are measured, evaluate `assign_patrols` in `crates/solver/src/algorithm/supply/state.rs`. Compare current count-based grouping to grouping based on patrol starting locations, refill likelihood, and supply-agent reach. Keep a valid baseline and account for the fact that supply and patrol routes are planned in separate stages.

## Tests and completion criteria

- A candidate whose required fuel exceeds available fuel is not ranked as executable unless a valid refill is planned in time.
- Arrival/slack calculations include fixed steps.
- Follow-up selection respects spot stock, per-day per-patrol spot visitation, and remaining time.
- More balls from a previously collected brand do not outrank obtaining an otherwise achievable distinct brand.
- Supply-assignment changes are evaluated separately from patrol target selection.
- The strategy improves or preserves official lexicographic score across the regression corpus without increasing invalid plans.
- Record runtime and planning behaviour; avoid heavier optimization if marginal score gains do not justify added complexity.
