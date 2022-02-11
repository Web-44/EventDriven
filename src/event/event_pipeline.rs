use std::collections::HashMap;
use crate::util::token::Token;

pub struct EventPipeline {
    pub listeners: HashMap<String, Vec<EventListener>>
}

pub struct EventListener {
    pub tokens: Vec<Token>
}