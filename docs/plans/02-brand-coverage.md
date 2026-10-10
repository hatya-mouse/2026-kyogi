# Phase 2: Improve distinct-brand coverage

## Goal

Allocate patrols to valuable unvisited brands as a team-wide decision, reducing cases where greedy candidate ordering assigns a flexible patrol to a brand that a less flexible patrol uniquely needs. Preserve iterative planning and A* route generation.

## Design

1. Extract the currently feasible patrol-to-brand options from reachable stocked spots. Retain candidate spot, estimated arrival time, route-step estimate, stock, and patrol/spot identity needed for execution.
2. Prioritize brands by scarcity: count distinct patrols with a feasible option, not raw spot count. A brand reachable by only one patrol is more constrained than one reachable by every patrol.
3. Select distinct patrol/brand pairs to maximize covered daily brands, then minimize assignment cost. Start with simple, interpretable costs such as arrival steps and slack, with deterministic tie-breaking. Avoid arbitrary weighted sums that can trade away a brand for a small distance gain.
4. For each matched brand, choose a feasible spot of that brand for its assigned patrol, preferring lower cost while respecting stock and one collection per patrol/spot/day.
5. Attempt selected assignments in an explicit order, update planning state on success, and recompute options. If route execution or feasibility fails, remove that failed option and rematch so its failure does not strand another patrol.
6. After distinct-brand opportunities are secured or exhausted, permit repeat-brand assignments for additional balls as a lower-priority phase.

Maximum-cardinality bipartite matching is a suitable first coverage implementation; evaluate min-cost maximum matching later if useful. Keep the objective lexicographic: first maximize feasible brand count, then minimize assignment cost. Do not use a weighted score unless proven bounds show it cannot sacrifice coverage.

## Important scoring caveat

The rules score match-wide unique brands before daily unique brands. Current planning state tracks only brands not yet collected today. Before optimizing match-wide novelty, add or pass authoritative match-history information from the manager/input layer if it exists. If no reliable history is supplied, state clearly that the phase optimizes daily distinct-brand coverage and uses it as a proxy; do not pretend it knows match novelty.

## Edge cases and tests

- Ignore brands with no feasible patrol/spot option.
- Several spots of one brand still count as one daily brand for coverage.
- One spot may have stock for multiple patrols, but each patrol may collect there at most once daily.
- If several patrols can reach only one shared brand, only one pair is needed for that brand before repeat-brand collection.
- A unique-access brand must not be lost because a flexible patrol has a slightly shorter route to a different brand.
- Equal-cost matching is deterministic.
- A failed execution attempt triggers rematching or a safe fallback without emitting an illegal plan.
- Compare with Phase 1 on identical fixtures and the recent match input if reconstructable.

## Implementation status

Implemented in the solver:

- A deterministic min-cost maximum-flow assignment maps patrols through candidate spots to brands. The primary objective is maximum daily distinct-brand count; only among maximum-cardinality assignments does it minimise lexicographic `(route steps, remaining-step slack, patrol ID, spot ID)` cost. No weighted sum is used.
- Stock and daily brand capacity are part of the flow network: each patrol can take one matched assignment, each spot is limited by remaining stock, and each currently unvisited brand has capacity one for coverage. Matched coverage candidates are attempted before remaining repeat-brand candidates, and existing iterative replanning continues after each successful collection.
- Execution failures exclude only that patrol/spot option before rematching. Refill-time recalculation uses the same exclusion/retry behavior for its patrol.
- Set `SOLVER_BRAND_COVERAGE=0` to restore the legacy ranked candidate strategy. The default enables the new assignment.
- `SOLVER_DISTRIBUTION_DIAGNOSTICS=1` remains available. The baseline fixture now also includes a validator-accepted unique-access case. Matching tests exercise cardinality, a flexible-versus-constrained patrol graph, stock/brand capacity, and deterministic candidate ordering.

## Validation and caveats

The solver test suite passes on the Phase 2 implementation, including the Phase 1 repeated-run baseline and the unique-access end-to-end fixture. This validates synthetic cases only; the recent multi-day match input and later-day statuses are not available as a replayable fixture, so competition improvement has not yet been established. Compare both modes on captured tournament inputs before treating the change as a proven score gain. A* route search, validator rules, and supply-planning policy were not changed.

The matcher optimises distinct brands available *today*, not match-wide novelty. Current solver input does not include authoritative match history. Assignment cost is deliberately lexicographic and deterministic, but this is not a full global travel-distance optimiser: it minimises the sum of listed candidate costs after coverage, while iterative planning regenerates candidates after each accepted collection.
