use crate::util::position::Position;

pub mod position;
pub mod token;
pub mod variables;
pub mod scope;
pub mod debug;

pub fn error(msg: String, pos: Position) {
    panic!("ERROR: {} (at {})", msg, pos);
}

pub fn error_unknown_pos(msg: String) {
    panic!("ERROR: {} (unknown position)", msg);
}

pub fn error_str(msg: &str, pos: Position) {
    panic!("ERROR: {} (at {})", msg, pos);
}

pub fn warning(msg: String, pos: Position) {
    eprintln!("WARNING: {} (at {})", msg, pos);
}

pub fn warning_str(msg: &str, pos: Position) {
    eprintln!("WARNING: {} (at {})", msg, pos);
}

pub fn runtime_error_str(msg: &str, pos: Position) {
    panic!("RUNTIME ERROR: {} (at {})", msg, pos);
}

pub fn runtime_error(msg: String, pos: Position) {
    panic!("RUNTIME ERROR: {} (at {})", msg, pos);
}

pub fn runtime_error_unknown_pos(msg: String) {
    panic!("RUNTIME ERROR: {} (unknown position)", msg);
}