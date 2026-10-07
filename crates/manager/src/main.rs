mod cli;
mod config;
mod game_api;

use crate::{
    cli::Cli,
    config::load_config,
    game_api::{Api, ApiActionPlanAnswer, ApiAgentKindAnswer},
};
use clap::Parser;
use owo_colors::OwoColorize;
use solver::{
    algorithm::Solver,
    game::{AgentKind, Brand, Map, Spot},
    tui::{println_error, println_info},
    validator::validate_day_plan,
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() {
    // Load environment variable
    dotenvy::dotenv().ok();

    // Parse the arguments
    let cli = Cli::parse();

    let Some(config) = load_config(&cli) else {
        return;
    };

    println_info("Successfully loaded the configuration file.");
    if cli.verbose {
        println_info("Loaded configuration:");
        println!(
            "{}: {}",
            "Game Server URL".green().bold(),
            config.game_server_url
        );
        println!(
            "{}: {}",
            "Game Token Env".green().bold(),
            config.game_token_env
        );
    }

    // Get the token and create an API
    let token = match std::env::var(&config.game_token_env) {
        Ok(token) => token,
        Err(err) => {
            println_error(format!(
                "Game token env \"{}\" could not be found:\n{}",
                config.game_token_env, err
            ));
            return;
        }
    };

    let api = match Api::new(config.game_server_url.clone(), token) {
        Ok(api) => api,
        Err(err) => {
            println_error(format!(
                "Could not initialize reqwest::Client with URL {}:\n{}",
                config.game_server_url, err
            ));
            return;
        }
    };

    // Get the initial setting from the game server
    let initial_data = loop {
        match api.get_initial().await {
            Ok(data) => break data,
            Err(err) => {
                println_error(format!("Error getting the initial setting data:\n{}", err));
            }
        }
        println_info(format!(
            "{} in {} seconds",
            "Retrying".yellow(),
            "5".green().bold()
        ));
        tokio::time::sleep(std::time::Duration::from_millis(5000)).await;
    };

    let supply_count = config
        .supply_count
        .min(initial_data.agents.len().saturating_sub(1));
    let mut agent_kinds = vec![AgentKind::Supply; initial_data.agents.len()];
    for kind in agent_kinds.iter_mut().skip(supply_count) {
        *kind = AgentKind::Patrol;
    }
    let agents_answer = ApiAgentKindAnswer(agent_kinds);
    if let Err(err) = api.post_agents(&agents_answer).await {
        println_error(format!("Error submitting the agent kind:\n{}", err));
        return;
    }
    println_info(format!("Submitted the agent kind:\n{:?}", agents_answer));

    let starts_at = initial_data.starts_at;
    let day_seconds = initial_data.day_seconds;
    let map = Map::new(
        (
            initial_data.map.width as usize,
            initial_data.map.height as usize,
        ),
        initial_data.map.cells.into_iter().flatten().collect(),
    );
    let spots = initial_data
        .spots
        .into_iter()
        .map(|spot| {
            (
                spot.pos,
                Spot::new(Brand::new(spot.brand as i64), spot.stocks),
            )
        })
        .collect();
    let solver = Solver::new(
        &map,
        spots,
        initial_data.agents.len(),
        initial_data.fuel_limits,
    );
    let day_steps = initial_data.day_steps;

    wait_until(starts_at).await;

    let mut previous_day = -1;
    let last_day = day_steps.len().saturating_sub(1) as i32;
    loop {
        let day = loop {
            match api.get_day().await {
                Ok(day) if day.day > previous_day => break day,
                Ok(_) => {
                    tokio::time::sleep(Duration::from_millis(200)).await;
                }
                Err(err) => {
                    println_error(format!("Error getting the day data:\n{}", err));
                    tokio::time::sleep(std::time::Duration::from_millis(2000)).await;
                }
            }
        };

        let Some(&steps) = day_steps.get(day.day as usize) else {
            println_error(format!("Received an invalid day index: {}", day.day));
            return;
        };

        println_info("================================");
        println_info(format!(
            "{} {} {}",
            "Day".green(),
            day.day.bold(),
            "has started".green()
        ));

        let day_ends_at = day.ends_at;
        let day_number = day.day;
        previous_day = day_number;
        let day_data = day.as_solver_day_data(steps);
        let plan = solver.solve_day(&day_data);

        if cli.verbose {
            for error in validate_day_plan(&map, &day_data, &plan, initial_data.fuel_limits) {
                println_error(format!(
                    "Invalid generated plan for agent {}: {}",
                    error.agent_id, error.message
                ));
            }
        }

        if cli.verbose {
            println_info(format!("Generated a plan for day {}:", day_data.day));

            for (agent_id, agent_actions) in plan.actions.iter().enumerate() {
                println!(
                    "- Agent {}: {:?}",
                    agent_id,
                    serde_json::to_string(agent_actions).unwrap_or("could not show".to_string())
                );
            }
        }

        match api.post_plan(&ApiActionPlanAnswer(plan.actions)).await {
            Ok(response) if response.revision >= 0 => {
                println_info(format!(
                    "Accepted the plan for day {} (revision {}).",
                    day_data.day, response.revision
                ));
            }
            Ok(response) => {
                println_error(format!(
                    "The plan for day {} was rejected (revision {}).",
                    day_data.day, response.revision
                ));
            }
            Err(err) => {
                println_error(format!("Error submitting the action plan:\n{}", err));
                continue;
            }
        }

        if day_data.day >= last_day {
            println_info("Match finished.");
            break;
        }

        let next_day_start = starts_at.saturating_add(
            day_seconds
                .iter()
                .take(day_number as usize + 1)
                .map(|seconds| *seconds as u64)
                .sum::<u64>(),
        );
        wait_until(next_day_start.max(day_ends_at)).await;
    }
}

async fn wait_until(timestamp: u64) {
    loop {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let Some(remaining) = timestamp.checked_sub(now) else {
            return;
        };
        if remaining == 0 {
            return;
        }

        tokio::time::sleep(Duration::from_secs(remaining.min(1))).await;
    }
}
