use substring::Substring;
use crate::event::event_pipeline::EventPipeline;

use crate::event::standard_events::{Event, get_event};
use crate::util::{runtime_error, runtime_error_str, runtime_error_unknown_pos};
use crate::util::debug::debug;
use crate::util::position::Position;
use crate::util::scope::Scope;
use crate::util::token::{Token, TokenType};
use crate::util::variables::{Variable, VariableType};

pub struct VM {
    pub pipeline: EventPipeline
}

impl VM {
    pub fn start(&mut self) {
        let scopes = &mut vec![Scope::default()];
        self.call_event(None, scopes, &String::from("OnStart"), &None);
    }

    fn call_event(&self, current_event: Option<(Position, &Box<dyn Event>)>, scopes: &mut Vec<Scope>, name: &String, params: &Option<Vec<String>>) -> Box<dyn Event> {
        if let Some(mut event) = get_event(name) {
            if let Some((pos, current_event)) = current_event {
                let mut count: u8 = 0;
                if let Some(params) = params {
                    params.iter().for_each(|param| {
                        let var = get_dynamic_value(name, current_event, param, pos, scopes);
                        if !event.accept(count, var) {
                            runtime_error(format!("Invalid event parameter ({}) for event {}", param, name), pos);
                        }
                        count += 1;
                    });
                }
                if !event.check_param_count(count) {
                    runtime_error(format!("Incorrect event parameter count for event {}", name), pos);
                }
            }
            event.call();
            if let Some(listeners) = self.pipeline.listeners.get(event.name()) {
                for listener in listeners {
                    let mut cancel = false;
                    listener.tokens.iter().for_each(|token| {
                        if self.execute_token(&event, scopes, &token) {
                            cancel = true;
                        }
                    });
                    if cancel {
                        break;
                    }
                }
            }

            event
        } else if let Some((pos,_)) = current_event {
            runtime_error(format!(r#"No such event: {}"#, name), pos);
            unreachable!()
        } else {
            runtime_error_unknown_pos(format!(r#"No such event: {}"#, name));
            unreachable!()
        }
    }

    fn execute_token(&self, current_event: &Box<dyn Event>, scopes: &mut Vec<Scope>, token: &Token) -> bool {
        debug(format!("EXECUTE: {:?}", token));
        match &token.token {
            TokenType::ScopeStart => {
                scopes.push(Scope::default());
            }
            TokenType::ScopeEnd => {
                scopes.pop();
                if scopes.is_empty() {
                    runtime_error_str("Global Scope dropped", token.pos);
                }
            }
            TokenType::InitVariable(name, typ, is_static_value, var_cast, value) => {
                if find_var(name, scopes).is_some() {
                    runtime_error(format!("Variable {} already exists in this scope", name), token.pos);
                }
                if let Some(value) = value {
                    let var = if *is_static_value {
                        Variable::new(name, typ, value, token.pos)
                    } else {
                        let value = get_dynamic_value(name, current_event, value, token.pos, scopes);
                        Variable::new_type(name, typ, *var_cast, value, token.pos)
                    };
                    if let Some(scope) = scopes.last_mut() {
                        scope.stack.push(var);
                    } else {
                        runtime_error(format!("Can't create variable {} (type {}) without a scope", name, typ), token.pos);
                    }
                } else {
                    runtime_error(format!("Missing initial value for variable {} (type {})", name, typ), token.pos);
                }
            }
            TokenType::VariableStaticSet(name, value) => {
                replace_var_static(name, value, token.pos, scopes);
            }
            TokenType::VariableDynamicSet(name, source, var_cast) => {
                if let Some((target_scope_index, target_var_index)) = find_var(name, scopes) {
                    let value = get_dynamic_value(name, current_event, source, token.pos, scopes);

                    let target_scope = scopes.get_mut(target_scope_index).unwrap();
                    let target_var = target_scope.stack.remove(target_var_index);
                    target_scope.stack.insert(target_var_index, Variable::copy_type(target_var, *var_cast, value, token.pos));
                } else {
                    runtime_error(format!(r#"Variable "{}" not found in current scope!"#, name), token.pos);
                }
            }
            TokenType::VariableEventSet(name, event, params) => {
                if let Some((target_scope_index, target_var_index)) = find_var(name, scopes) {
                    let event = self.call_event(Some((token.pos, current_event)), scopes, event, params);

                    let target_scope = scopes.get_mut(target_scope_index).unwrap();
                    target_scope.stack.remove(target_var_index);

                    target_scope.stack.insert(target_var_index, Variable::new_event_type(name, event));
                } else {
                    runtime_error(format!(r#"Variable "{}" not found in current scope!"#, name), token.pos);
                }
            }
            TokenType::CallEvent(name, params) => {
                self.call_event(Some((token.pos, current_event)), scopes, name, params);
            }
            TokenType::InitVariableEvent(var_name, name, params) => {
                let event = self.call_event(Some((token.pos, current_event)), scopes, name, params);
                if let Some(scope) = scopes.last_mut() {
                    scope.stack.push(Variable::new_event_type(var_name, event));
                } else {
                    runtime_error(format!("Can't create variable {} (type event) without a scope", name), token.pos);
                }
            }
            TokenType::Raw(s) => {
                runtime_error(format!("Tried to execute unparsed instruction: {}", s), token.pos);
            }
            _ => {}
        }
        debug(format!("CURRENT SCOPES: {:?}", scopes));
        false
    }
}

fn get_dynamic_value(name: &String, current_event: &Box<dyn Event>, source: &String, pos: Position, scopes: &Vec<Scope>) -> VariableType {
    if current_event.name() == source.as_str() {
        todo!()
    } else if let Some(var) = get_var_value(source, scopes) {
        if let VariableType::Event(event) = var {
            if let Some(var) = event.get_var(name) {
                var
            } else {
                runtime_error(format!(r#"Event parameter {} not found in event {}"#, name, current_event.name()), pos);
                unreachable!()
            }
        } else {
            var
        }
    } else if source.starts_with('"') && source.ends_with('"') {
        VariableType::String(Some(source.substring(1, source.len() - 1).to_string()))
    } else {
        runtime_error(format!(r#"Invalid dynamic source for {}: {}"#, name, source), pos);
        unreachable!()
    }
}

fn replace_var_static(name: &String, value: &String, pos: Position, scopes: &mut Vec<Scope>) {
    if let Some((scope_index, var_index)) = find_var(name, scopes) {
        let scope = scopes.get_mut(scope_index).unwrap();
        let var = scope.stack.remove(var_index);
        scope.stack.insert(var_index, Variable::copy(var, value, pos));
    } else {
        runtime_error(format!(r#"Variable "{}" not found in current scope!"#, name), pos);
    }
}

fn find_var(name: &String, scopes: &Vec<Scope>) -> Option<(usize, usize)> {
    let mut scope_index = 0;
    for scope in scopes {
        let mut var_index = 0;
        for var in &scope.stack {
            if var.name.eq(name) {
                return Some((scope_index, var_index));
            }
            var_index += 1;
        }
        scope_index += 1;
    }
    None
}

fn get_var_value(name: &String, scopes: &Vec<Scope>) -> Option<VariableType> {
    for scope in scopes {
        for var in &scope.stack {
            if var.name == *name {
                return Some(var.variable_type.clone());
            }
        }
    }
    None
}