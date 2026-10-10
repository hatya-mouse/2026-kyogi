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

## Implementation status

Implemented in the patrol assignment path:

- Cache route-step and patrol-fuel estimates per origin/destination using the existing A* route and the planner's movement terrain costs. Do not assume refills before supply planning.
- Exclude candidates when that existing A* route exceeds current fuel or when `fixed_steps + route_steps` exceeds the day. Compute slack from the actual arrival step; final action insertion and plan validation remain authoritative.
- Add bounded one-stop follow-up evaluation. Rank feasible next stops that add a still-uncollected daily brand above other feasible collections, then use feasible follow-up work as a lower-priority signal. Rebuild candidate values on each planning iteration as stock and daily brand state change.
- Preserve maximum distinct-brand matching before applying follow-up, route effort, fuel, and deterministic tie-break costs. Supply-agent grouping and A* itself are unchanged.
- Add unit and integration coverage for fuel-infeasible routes, fixed-step arrival, follow-up step/fuel/stock/visited-spot constraints, daily marginal brand value, and a valid lookahead-selected plan.

Validation run in WSL on the current worktree:

- `cargo fmt --all -- --check` passed.
- `cargo test -p solver` passed: 7 unit tests and 2 integration tests.
- `cargo check --workspace` passed.
- `git diff --check` passed.

The documented fixed-runtime corpus could not be run: `scripts/validate_fixed_runtime.py` is absent from this checkout (the README refers to it), and the attempted `python` command is not installed; `python3` is available, but the script is still missing. No before/after multi-day score or runtime comparison has therefore been measured, and this implementation is not yet demonstrated to improve or preserve competition scores across the regression corpus. The checked-in finals fixtures also do not include later-day statuses.

Fuel feasibility is currently measured on the single route returned by the unchanged step-oriented A*. A different, slower route could theoretically use less fuel; in that case this filter is conservative relative to the existing route generator, not a proof that no physical route can reach the candidate. Supporting such alternatives would require route-selection changes and remains outside this phase's scope. No future refill is presumed, and final route/action feasibility checks are retained.
