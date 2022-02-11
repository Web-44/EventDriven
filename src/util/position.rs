use std::fmt::{Display, Formatter};

#[derive(Default, Copy, Clone, Debug)]
pub struct Position {
    pub(crate) line: u16,
    pub(crate) index: u16
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.index)
    }
}