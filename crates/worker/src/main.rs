use crate::{cli::Cli, config::WorkerConfig};
use clap::Parser;
use owo_colors::OwoColorize;
use solver::tui::{println_error, println_info};
use std::path::{Path, PathBuf};

mod cli;
mod config;

fn main() {
    let cli = Cli::parse();

    // Load the config file
    let config_path = PathBuf::from(&cli.config);
    let config_file = match load_config(&config_path) {
        Ok(path) => path,
        Err(err) => {
            println_error(format!(
                "An error occured while loading config file from {}:\n{}",
                cli.config, err
            ));
            return;
        }
    };

    // Deserialize the config
    let config: WorkerConfig = match serde_json::from_str(&config_file) {
        Ok(config) => config,
        Err(err) => {
            println_error(format!(
                "An error occured while parsing config file:\n{}",
                err
            ));
            return;
        }
    };

    println_info("Successfully loaded the configuration file.");
    if cli.verbose {
        println_info("Loaded configuration:");
        println!("{}: {}", "Port".green().bold(), config.port);
    }
}

fn load_config(config_path: &Path) -> std::io::Result<String> {
    // Get an absolute path to the file
    let absolute_config_path = config_path.canonicalize()?;
    // Get the content of the config file
    std::fs::read_to_string(absolute_config_path)
}
