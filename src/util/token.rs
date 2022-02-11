use crate::util::position::Position;

#[derive(Clone, Debug)]
pub struct Token {
    pub token: TokenType,
    pub pos: Position
}

#[derive(PartialEq, Clone, Debug)]
pub enum TokenType {
    Keyword(Keyword),
    Listener(String),
    CallEvent(String, Option<Vec<String>>),
    ScopeStart,
    ScopeEnd,
    Raw(String),
    InitVariable(String, String, bool, bool, Option<String>),
    InitVariableEvent(String, String, Option<Vec<String>>),
    VariableStaticSet(String, String),
    VariableDynamicSet(String, String, bool),
    VariableEventSet(String, String, Option<Vec<String>>)
}

#[derive(PartialEq, Clone, Debug)]
pub enum Keyword {
}