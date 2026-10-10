# Agent distribution improvement plan

This plan improves patrol-agent task allocation while leaving the existing A* route search unchanged. Work is split into ordered phases so each can be implemented and evaluated independently.

## Phases

1. [Measure current allocation](01-measure-baseline.md) — add diagnostics and a repeatable baseline.
2. [Improve distinct-brand coverage](02-brand-coverage.md) — globally assign scarce, unvisited brands instead of relying only on a sorted greedy candidate list.
3. [Improve follow-up work and resource awareness](03-follow-up-and-resources.md) — select short-horizon follow-up work while accounting for fuel, timing, and supply support.

## Objectives and constraints

Follow `docs/rules/05_scoring.md`: maximize match-wide distinct brands first, daily distinct brands second, and total balls third. Current `PlanningState::unvisited_brands` tracks brands unvisited today, but inspected `DayData` and `PlanningState` do not expose match-wide collection history. Do not claim to optimize match-wide novelty until authoritative history is available or a documented proxy is chosen.

Respect daily spot stock, at most one successful collection by a patrol at a given spot each day, movement-time limits, fuel, fixed actions, and refill planning. Keep A* route generation unchanged unless experiments reveal an allocation-side integration defect.

## Working method

Execute one phase at a time. Compare variants on identical inputs, in official objective order, and record fixtures, configuration, code revision, results, and regressions. Do not tune weights against only one match.

## Current implementation observations

`crates/solver/src/algorithm/distribution/mod.rs` builds agent/spot candidates, uses stock-aware maximum matching to reserve daily distinct-brand opportunities, then repeatedly accepts a feasible assignment and regenerates candidates after state changes. Phase 3 adds cached A*-route step/fuel estimates, fixed-step-aware feasibility and slack, plus bounded one-stop follow-up scoring. Supply-agent grouping in `crates/solver/src/algorithm/supply/state.rs` remains count-based and is deliberately out of scope.

Unit and integration tests cover synthetic allocation cases, and the fixed-runtime validation remains outstanding because its documented runner is absent from this checkout. The recent match log shows patrols with no meaningful route, but does not prove those agents had useful feasible work available; no official score improvement is claimed without reproducible multi-day inputs and comparison.
