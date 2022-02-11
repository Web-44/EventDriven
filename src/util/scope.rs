use crate::util::variables::Variable;

#[derive(Default,Clone,Debug)]
pub struct Scope {
    pub stack: Vec<Variable>
}