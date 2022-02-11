#[cfg(debug_assertions)]
pub fn debug(msg: String) {
    println!("DEBUG: {}", msg);
}

#[cfg(debug_assertions)]
pub fn debug_str(msg: &str) {
    println!("DEBUG: {}", msg);
}

#[cfg(not(debug_assertions))]
pub fn debug(_msg: String) {
}

#[cfg(not(debug_assertions))]
pub fn debug_str(_msg: &str) {
}