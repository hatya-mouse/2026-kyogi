use super::{CostMap, PlanningState, Solver, distribution::RouteCostCache};
use crate::{
    game::{Agent, AgentKind, Brand, CellId, CellType, DayData, Map, Spot, TrafficStatus},
    validator::validate_day_plan,
};
use std::collections::HashMap;

fn baseline_fixture() -> (Map, HashMap<CellId, Spot>, DayData) {
    let map = Map::new(
        (4, 2),
        vec![
            CellType::Plain,
            CellType::Road,
            CellType::Plain,
            CellType::Plain,
            CellType::Plain,
            CellType::Mountain,
            CellType::Plain,
            CellType::Plain,
        ],
    );
    let spots = HashMap::from([
        (CellId(2), Spot::new(Brand::new(10), 1)),
        (CellId(3), Spot::new(Brand::new(20), 2)),
        (CellId(7), Spot::new(Brand::new(30), 1)),
    ]);
    let agents = vec![
        Agent {
            kind: AgentKind::Patrol,
            pos: CellId(0),
            fuel: 8,
        },
        Agent {
            kind: AgentKind::Patrol,
            pos: CellId(4),
            fuel: 8,
        },
        Agent {
            kind: AgentKind::Supply,
            pos: CellId(6),
            fuel: 0,
        },
    ];
    let traffics = HashMap::from([(CellId(1), TrafficStatus::Smooth)]);
    let day = DayData {
        day: 0,
        steps: 8,
        agents,
        traffics,
    };

    (map, spots, day)
}

#[test]
fn brand_coverage_preserves_unique_access_and_validates_plan() {
    let map = Map::new(
        (3, 2),
        vec![
            CellType::Plain,
            CellType::Plain,
            CellType::Plain,
            CellType::Plain,
            CellType::Plain,
            CellType::Plain,
        ],
    );
    let spots = HashMap::from([
        (CellId(1), Spot::new(Brand::new(10), 1)),
        (CellId(3), Spot::new(Brand::new(20), 1)),
    ]);
    let day = DayData {
        day: 0,
        steps: 5,
        agents: vec![
            Agent {
                kind: AgentKind::Patrol,
                pos: CellId(0),
                fuel: 8,
            },
            Agent {
                kind: AgentKind::Patrol,
                pos: CellId(2),
                fuel: 8,
            },
            Agent {
                kind: AgentKind::Supply,
                pos: CellId(5),
                fuel: 0,
            },
        ],
        traffics: HashMap::new(),
    };
    let solver = Solver::new(&map, spots, day.agents.len(), 8);
    let plan = solver.solve_day(&day);
    assert_eq!(plan.actions.len(), day.agents.len());
    assert_eq!(
        format!("{:?}", &plan.actions[..2]).matches("Move").count(),
        2
    );
    assert!(validate_day_plan(&map, &day, &plan, 8).is_empty());
}

#[test]
fn baseline_fixture_is_deterministic_and_valid() {
    let (map, spots, day) = baseline_fixture();
    let solver = Solver::new(&map, spots, day.agents.len(), 8);
    let first = solver.solve_day(&day);
    let second = solver.solve_day(&day);

    assert_eq!(
        format!("{:?}", &first.actions[..2]),
        format!("{:?}", &second.actions[..2])
    );
    assert!(validate_day_plan(&map, &day, &first, 8).is_empty());
    assert!(validate_day_plan(&map, &day, &second, 8).is_empty());
}

#[test]
fn candidate_arrival_includes_existing_fixed_steps() {
    let map = Map::new((2, 1), vec![CellType::Plain, CellType::Plain]);
    let spots = HashMap::from([(CellId(1), Spot::new(Brand::new(10), 1))]);
    let day = DayData {
        day: 0,
        steps: 5,
        agents: vec![Agent {
            kind: AgentKind::Patrol,
            pos: CellId(0),
            fuel: 8,
        }],
        traffics: HashMap::new(),
    };
    let solver = Solver::new(&map, spots, day.agents.len(), 8);
    let mut state = PlanningState::from_day(&solver.board, &day);
    state.cursor[0].fixed_steps = 4;
    let cost_map = CostMap::build(solver.board.map, &day);
    let assignments = solver.distribute_agents_excluding(
        &day,
        &state,
        &cost_map,
        &std::collections::HashSet::new(),
        &mut RouteCostCache::default(),
    );

    assert!(assignments.is_empty());
}

#[test]
fn follow_up_respects_remaining_steps_fuel_stock_and_daily_brands() {
    let map = Map::new(
        (3, 1),
        vec![CellType::Plain, CellType::Plain, CellType::Plain],
    );
    let spots = HashMap::from([
        (CellId(1), Spot::new(Brand::new(10), 1)),
        (CellId(2), Spot::new(Brand::new(20), 1)),
    ]);
    let day = DayData {
        day: 0,
        steps: 8,
        agents: vec![Agent {
            kind: AgentKind::Patrol,
            pos: CellId(0),
            fuel: 8,
        }],
        traffics: HashMap::new(),
    };
    let solver = Solver::new(&map, spots, day.agents.len(), 8);
    let state = PlanningState::from_day(&solver.board, &day);
    let cost_map = CostMap::build(solver.board.map, &day);
    let mut cache = RouteCostCache::default();
    let no_visited_spots = std::collections::HashSet::new();

    let feasible = solver.follow_up_value(
        &day,
        &cost_map,
        &mut cache,
        CellId(1),
        2,
        1,
        &state.spots,
        &no_visited_spots,
        &state.unvisited_brands,
    );
    assert!(feasible.collection);
    assert!(feasible.new_brand);

    let too_little_time = solver.follow_up_value(
        &day,
        &cost_map,
        &mut cache,
        CellId(1),
        1,
        1,
        &state.spots,
        &no_visited_spots,
        &state.unvisited_brands,
    );
    assert!(!too_little_time.collection);

    let too_little_fuel = solver.follow_up_value(
        &day,
        &cost_map,
        &mut cache,
        CellId(1),
        2,
        0,
        &state.spots,
        &no_visited_spots,
        &state.unvisited_brands,
    );
    assert!(!too_little_fuel.collection);

    let mut exhausted_stock = state.spots.clone();
    exhausted_stock.get_mut(&CellId(2)).unwrap().stocks = 0;
    let no_stock = solver.follow_up_value(
        &day,
        &cost_map,
        &mut cache,
        CellId(1),
        2,
        1,
        &exhausted_stock,
        &no_visited_spots,
        &state.unvisited_brands,
    );
    assert!(!no_stock.collection);

    let mut already_visited = std::collections::HashSet::new();
    already_visited.insert(CellId(2));
    let visited = solver.follow_up_value(
        &day,
        &cost_map,
        &mut cache,
        CellId(1),
        2,
        1,
        &state.spots,
        &already_visited,
        &state.unvisited_brands,
    );
    assert!(!visited.collection);

    let mut brands_already_collected = state.unvisited_brands.clone();
    brands_already_collected.remove(&Brand::new(20));
    let no_new_daily_brand = solver.follow_up_value(
        &day,
        &cost_map,
        &mut cache,
        CellId(1),
        2,
        1,
        &state.spots,
        &no_visited_spots,
        &brands_already_collected,
    );
    assert!(no_new_daily_brand.collection);
    assert!(!no_new_daily_brand.new_brand);
}
