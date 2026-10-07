use chrono::Local;
use owo_colors::OwoColorize;
use std::fmt::Display;

pub fn println_error(message: impl Display) {
    let formatted = Local::now().format("%Y-%m-%d %H:%M:%S %Z");
    eprintln!("{} {} {}", formatted, " Error ".on_red(), message);
}

pub fn println_info(message: impl Display) {
    let formatted = Local::now().format("%Y-%m-%d %H:%M:%S %Z");
    eprintln!("{} {} {}", formatted, " Info ".on_green(), message);
}
