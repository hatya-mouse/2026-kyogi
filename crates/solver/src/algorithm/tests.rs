use super::Solver;
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
