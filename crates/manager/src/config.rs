use serde::Deserialize;
use solver::{file::read_from_relative_path, tui::println_error};
use std::path::PathBuf;

use crate::Cli;

/// The configuration for the manager.
#[derive(Deserialize, Debug)]
pub(crate) struct ManagerConfig {
    /// The URL of the game server.
    pub game_server_url: String,
    /// The name of the environment variable where the token is stored.
    pub game_token_env: String,
    /// Number of agents that should be assigned as supply cars.
    #[serde(default)]
    pub supply_count: usize,
}

pub(super) fn load_config(cli: &Cli) -> Option<ManagerConfig> {
    // Load the config file
    let config_path = PathBuf::from(&cli.config);
    let config_file = match read_from_relative_path(&config_path) {
        Ok(path) => path,
        Err(err) => {
            println_error(format!(
                "An error occured while loading config file from {}:\n{}",
                cli.config, err
            ));
            return None;
        }
    };

    // Deserialize the config
    match serde_json::from_str(&config_file) {
        Ok(config) => Some(config),
        Err(err) => {
            println_error(format!(
                "An error occured while parsing config file:\n{}",
                err
            ));
            None
        }
    }
}
