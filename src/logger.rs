use std::fmt::Display;

pub fn info(message: impl Display) {
    println!("[info] {}", message);
}

pub fn error(message: impl Display) {
    println!("[err] {}", message);
}
