use solver::{
    algorithm::Solver,
    game::{
        Action, Agent, AgentKind, Brand, CellId, CellType, DayData, Direction, Map, Spot,
        TrafficStatus,
    },
    validator::validate_day_plan,
};
use std::collections::HashMap;

fn patrol(pos: usize, fuel: u32) -> Agent {
    Agent {
        kind: AgentKind::Patrol,
        pos: CellId(pos),
        fuel,
    }
}

fn day(steps: u32, agents: Vec<Agent>, traffics: HashMap<CellId, TrafficStatus>) -> DayData {
    DayData {
        day: 0,
        steps,
        agents,
        traffics,
    }
}

#[test]
fn excludes_a_route_that_exceeds_current_fuel() {
    let map = Map::new((2, 1), vec![CellType::Mountain, CellType::Plain]);
    let spots = HashMap::from([(CellId(1), Spot::new(Brand::new(10), 1))]);
    let day = day(3, vec![patrol(0, 1)], HashMap::new());
    let solver = Solver::new(&map, spots, day.agents.len(), 8);

    let plan = solver.solve_day(&day);

    assert!(matches!(plan.actions[0].as_slice(), [Action::Wait(steps)] if steps.get() == 3));
    assert!(validate_day_plan(&map, &day, &plan, 8).is_empty());
}

#[test]
fn lookahead_selects_a_first_spot_that_enables_a_distinct_brand_follow_up() {
    let map = Map::new(
        (3, 2),
        vec![
            CellType::Plain,
            CellType::Plain,
            CellType::Pond,
            CellType::Pond,
            CellType::Road,
            CellType::Pond,
        ],
    );
    let spots = HashMap::from([
        (CellId(1), Spot::new(Brand::new(20), 1)),
        (CellId(4), Spot::new(Brand::new(10), 1)),
    ]);
    let day = day(
        3,
        vec![patrol(0, 3)],
        HashMap::from([(CellId(4), TrafficStatus::Smooth)]),
    );
    let solver = Solver::new(&map, spots, day.agents.len(), 8);

    let plan = solver.solve_day(&day);

    assert!(matches!(
        plan.actions[0].as_slice(),
        [
            Action::Move(Direction::BottomRight),
            Action::Move(Direction::TopRight)
        ]
    ));
    assert!(validate_day_plan(&map, &day, &plan, 8).is_empty());
}
