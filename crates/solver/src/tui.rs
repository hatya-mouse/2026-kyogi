use chrono::Local;
use owo_colors::OwoColorize;
use std::fmt::Display;

pub fn println_error(message: impl Display) {
    eprintln!("{} {} {}", Local::now(), " Error ".on_red(), message);
}

pub fn println_info(message: impl Display) {
    eprintln!("{} {} {}", Local::now(), " Info ".on_green(), message);
}
