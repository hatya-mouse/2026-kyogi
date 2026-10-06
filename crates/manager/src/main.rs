mod cli;
mod config;
mod game_api;

use crate::{
    cli::Cli,
    config::load_config,
    game_api::{Api, ApiAgentKindAnswer},
};
use clap::Parser;
use owo_colors::OwoColorize;
use solver::{
    game::AgentKind,
    tui::{println_error, println_info},
};
use std::thread;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let Some(config) = load_config(&cli) else {
        return;
    };

    println_info("Successfully loaded the configuration file.");
    if cli.verbose {
        println_info("Loaded configuration:");
        println!("{}: {}", "Port".green().bold(), config.port);
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
        println!("Workers:");
        for (id, worker) in config.workers {
            println!(
                "- {} {:3}: {}:{}",
                "Worker".green().bold(),
                id.0.bold(),
                worker.address,
                worker.port
            );
        }
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
                println_error(format!("Error getting the setting data:\n{}", err));
            }
        }
        println_info(format!("{}", "Retrying...".yellow()));
        thread::sleep(std::time::Duration::from_millis(1000));
    };

    if let Err(err) = api
        .post_agents(&ApiAgentKindAnswer(vec![
            AgentKind::Patrol;
            initial_data.agents.len()
        ]))
        .await
    {
        println_error(format!("Error submitting the agent kind:\n{}", err));
        return;
    }
}
