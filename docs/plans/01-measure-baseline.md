# Phase 1: Measure current allocation

## Goal

Build a trustworthy baseline and diagnostics that explain why each patrol receives useful work, waits, or stops receiving assignments. Do not change allocation decisions in this phase except for fixes required to expose diagnostics safely.

## Inspect and instrument

- Trace `Solver::solve_day`, `distribute_agents`, `try_add_assignment`, and the transitions in `PlanningState`.
- For each planning iteration, record per patrol: current position, fixed steps, remaining day steps, fuel, candidate-spot count, reachable unvisited brands, and selected candidate.
- Record rejection/termination reasons where observable: no stock, already visited, unreachable, exceeds remaining steps, route conversion failure, fuel rejection, or no candidate remaining. Keep diagnostics optional or behind the existing logging conventions.
- Record the final patrol summary: distinct brands, collected spots/balls if derivable, movement steps, waits, and remaining fuel. Do not equate route length with score.
- Check whether plan-generation inputs include match-wide acquired-brand history. If absent, document where that information could be sourced and whether adding it affects solver interfaces.

## Baseline fixtures and measurement

- Identify deterministic historical inputs or build small synthetic cases covering: one scarce brand reachable by one patrol; several patrols competing for one-stock spots; distinct brands with different access; low-fuel patrols; limited day steps; and no remaining useful spots.
- Run the current solver repeatedly on identical inputs and confirm deterministic output, or record any intentional nondeterminism.
- Capture outcomes in official lexicographic order: match-wide distinct brands when known, daily distinct brands, then total balls. Also track secondary diagnostics such as failed candidate attempts and idle patrols with feasible candidates.
- Preserve validator/test behaviour. Add tests at the nearest existing solver test boundary; if none exists, create focused unit tests in the relevant module without expanding the work into unrelated test infrastructure.

## Phase 1 implementation status

Implemented in the solver:

- Set `SOLVER_DISTRIBUTION_DIAGNOSTICS=1` to emit per-iteration candidate counts and retained spots, patrol position/steps/fuel, reachable-within-day unvisited brands, selected assignments, rejection reasons, termination, final collection/movement/wait/fuel summaries, and daily brand/ball totals.
- Candidate scoring, stock pruning, A* routing, validation, and normal-mode candidate generation are unchanged. Fuel is not treated as part of the candidate count; candidates reported as reachable-within-day may still fail insertion due to fuel or other constraints.
- Added `algorithm::tests::baseline_fixture_is_deterministic_and_valid`, matching `data/baseline/map.json` and `status.json`, which solves twice, checks that patrol action plans match, and validates both outputs.

Run the named fixture with `SOLVER_DISTRIBUTION_DIAGNOSTICS=1 cargo test -p solver baseline_fixture_is_deterministic_and_valid -- --nocapture`. On the current revision the fixture reports 3 distinct brands in assigned patrol spot sets, 3 distinct patrol-spot assignments in final cursors, and 4 successful assignment insertions, with no route, fuel, or step rejections. Candidate pruning lets the first patrol reserve one-stock spots. Later, both patrols can be assigned to the remaining shared-stock spot even though only the first reaches it; the other is reported as selected but absent from its final cursor. Assignment-insertion counts therefore are not confirmed collection counts and may overstate actual collections.

The repeated run produced identical patrol plans, but the supply plan's equal-cost route varied between runs. Treat this as intentional nondeterminism in the baseline until supply tie-breaking is separately investigated; this phase does not change it. This is a single synthetic-size day-0 fixture, not a multi-day replay, and does not exercise low-fuel failure, scarce-brand access across distinct patrols, or server-side state transitions. `data/finals/` has no later-day statuses. `data/baseline/history_empty.json` confirms an empty-history example, but the solver's `DayData` API has no match-wide acquired-brand history, so match-wide distinct-brand score is not measurable from current solver inputs. The match client would need to retain/read that history and pass it to the solver if match-wide novelty is to be optimized or reported.

## Completion criteria

- A repeatable baseline can be run against a named set of fixtures.
- Diagnostics distinguish “idle because no feasible target” from “idle despite feasible targets”.
- Baseline metrics are saved with the code revision and inputs.
- No change to route search or intended plan behaviour.

## Handoff

Phase 2 now uses a stock-aware, lexicographic min-cost maximum matching to reserve daily distinct-brand opportunities before repeat-brand assignments. It can be disabled with `SOLVER_BRAND_COVERAGE=0`. The strategy has synthetic regression coverage but still needs an apples-to-apples replay of captured competition inputs before claiming a match-score improvement. If most idle patrols have no feasible target, prioritize feasibility/resource modelling rather than further tuning brand priorities.
