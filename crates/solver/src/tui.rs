use owo_colors::OwoColorize;
use std::fmt::Display;

pub fn println_error(message: impl Display) {
    eprintln!("{} {}", " Error ".on_red(), message);
}

pub fn println_info(message: impl Display) {
    eprintln!("{} {}", " Info ".on_green(), message);
}
