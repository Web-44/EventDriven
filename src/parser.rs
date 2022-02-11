use std::collections::HashMap;
use crate::event::event_pipeline::EventListener;

use crate::util::{error, error_str, error_unknown_pos, warning};
use crate::util::debug::debug;
use crate::util::position::Position;
use crate::util::token::{Token, TokenType};

pub fn pre_parse(lines: Vec<String>) -> Vec<(Position, String)> {
    let lines = lines.iter().fold((0 as u16, vec![]), |acc, line| {
        let (mut line_number, mut list) = acc;

        line_number += 1;
        list.push((line_number, line));

        (line_number, list)
    }).1;
    let (unclosed, _, pos, list) = lines.iter().map(|line| {
        let (line_number, line) = line;

        line.split_whitespace().fold(vec![], |mut list: Vec<(Position, String)>, instr| {
            let mut len = list.len();
            len += list.iter().fold(0, |len, item| len + item.1.len());

            list.push((Position {
                line: *line_number,
                index: len as u16
            }, instr.to_string()));

            list
        })
    }).flatten().fold((false, String::new(), Position::default(), vec![]), |acc, instr| {
            let (mut in_string, mut s, mut pos, mut list) = acc;
            let (instr_pos, mut instr) = instr;

            let end_semicolon = instr.ends_with(';');
            if end_semicolon {
                instr.remove(instr.len() - 1);
            }

            if in_string {
                if instr.ends_with(r#"""#) {
                    s.push_str(instr.as_str());
                    if end_semicolon {
                        s.push(';');
                    }
                    list.push((pos, s));
                    s = String::new();
                    pos = Position::default();
                    in_string = false;
                }
                s.push_str(instr.as_str());
                if end_semicolon {
                    s.push(';');
                }
                s.push(' ');
            } else {
                if instr.starts_with(r#"""#) {
                    instr.remove(0);
                    pos = instr_pos;
                    if instr.ends_with(r#"""#) {
                        instr.insert(0, '"');
                        if end_semicolon {
                            instr.push(';');
                        }
                        list.push((pos, instr));
                    } else {
                        in_string = true;
                        s.push('"');
                        s.push_str(instr.as_str());
                        if end_semicolon {
                            s.push(';');
                        }
                        s.push(' ');
                    }
                } else if end_semicolon {
                    list.push((instr_pos, instr + ";"));
                } else {
                    list.push((instr_pos, instr));
                }
            }

            (in_string, s, pos, list)
        });

    if unclosed {
        error_str("String not closed", pos);
    }

    list
}

pub fn tokenize(instructions: Vec<(Position, String)>) -> Vec<Token> {
    let (scope_depth, _, _, _, _, _, list)
            = instructions.iter().fold((0, false, false, false, None, vec![], vec![]), |acc: (i32, bool, bool, bool, Option<String>, Vec<String>, Vec<Token>), instr| {
        let (mut scope_depth, mut var_static_set, mut var_dynamic_set, mut var_cast, mut listener_type, mut event_params, mut list) = acc;
        let (pos,mut instr) = instr.clone();

        let mut end_command_semicolon = instr.ends_with(';');
        if end_command_semicolon {
            instr.remove(instr.len() - 1);
        }

        let mut parsed = false;
        if let Some(_) = listener_type.clone() {
            if instr.starts_with('#') {
                instr.remove(0);
                if instr.is_empty() {
                    error_str("No event to call specified", pos);
                }

                if var_dynamic_set {
                    if let Some(tok) = list.pop() {
                        if let TokenType::InitVariable(name, typ, _, _, _) = tok.token {
                            if typ.as_str() != "event" {
                                error(format!("Can't pipe event to variable of type {}", typ), pos);
                            }
                            list.push(Token {
                                token: TokenType::InitVariableEvent(name, instr.clone(), None),
                                pos
                            });
                            var_dynamic_set = false;
                        } else if let TokenType::Raw(name) = tok.token {
                            list.push(Token {
                                token: TokenType::VariableEventSet(name, instr.clone(), None),
                                pos
                            });
                            var_dynamic_set = false;
                        } else {
                            error_str("Illegal event call (syntax doesn't make sense)", pos);
                        }
                    } else {
                        error_str("Can't pipe event to empty variable", pos);
                    }
                } else {
                    list.push(Token {
                        token: TokenType::CallEvent(instr.clone(), None),
                        pos
                    });
                }
                debug(format!("call event {}", instr));

                if end_command_semicolon {
                    end_command_semicolon = false;
                }
                parsed = true;
            } else if instr.starts_with('(') && instr.ends_with(')') {
                instr.remove(0);
                instr.remove(instr.len() - 1);
                if let Some(tok) = list.pop() {
                    if let TokenType::Raw(name) = tok.token {
                        debug(format!("variable {} of type {}", name, instr.clone()));
                        list.push(Token {
                            token: TokenType::InitVariable(name, instr.clone(), true, false, None),
                            pos
                        });
                        parsed = true;
                    } else {
                        error_str("No variable name specified", pos);
                    }
                } else {
                    error_str("No variable name specified", pos);
                }
            } else if instr == "=" {
                if let Some(tok) = list.last() {
                    if let TokenType::Raw(_) = &tok.token {
                    } else if let TokenType::InitVariable(_, _, _, _, _) = &tok.token {
                    } else {
                        error_str("No variable name specified", pos);
                    }
                } else {
                    error_str("No variable name specified", pos);
                }
                var_static_set = true;
                parsed = true;
            } else if instr == "<-" || instr == "<=" {
                if let Some(tok) = list.last() {
                    if let TokenType::Raw(_) = &tok.token {
                    } else if let TokenType::InitVariable(_, _, _, _, _) = &tok.token {
                    } else if let TokenType::CallEvent(_, _) = &tok.token {
                    } else if let TokenType::InitVariableEvent(_, _, _) = &tok.token {
                    } else if let TokenType::VariableEventSet(_, _, _) = &tok.token {
                    } else {
                        error_str("No dynamic target specified", pos);
                    }
                } else {
                    error_str("No dynamic target specified", pos);
                }
                var_cast = instr == "<=";
                var_dynamic_set = true;
                parsed = true;
            } else if instr == "{" {
                list.push(Token {
                    token: TokenType::ScopeStart,
                    pos
                });
                scope_depth += 1;
                debug(format!("entered scope (now level {})", scope_depth));
                parsed = true;
            } else if instr == "}" {
                list.push(Token {
                    token: TokenType::ScopeEnd,
                    pos
                });
                scope_depth -= 1;
                if scope_depth < 0 {
                    error(format!("Tried to exit non-existent scope (scope depth: {})", scope_depth), pos);
                }
                debug(format!("exited scope (now level {})", scope_depth));
                parsed = true;
            } else {
                if var_static_set {
                    if let Some(tok) = list.pop() {
                        if let TokenType::Raw(name) = &tok.token {
                            debug(format!("change variable {} to {}", name, instr.clone()));
                            list.push(Token {
                                token: TokenType::VariableStaticSet(name.clone(), instr.clone()),
                                pos
                            });
                        } else if let TokenType::InitVariable(name, typ, _, _, value) = &tok.token {
                            if value.is_some() {
                                error(format!(r#"Can't initiate variable "{}" twice!"#, name), pos);
                            }
                            debug(format!("initiate variable {} of type {} to {}", name, typ, instr.clone()));
                            list.push(Token {
                                token: TokenType::InitVariable(name.clone(), typ.clone(), true, false, Some(instr.clone())),
                                pos
                            });
                        } else {
                            error_str("No variable name specified", pos);
                        }
                    }
                    if end_command_semicolon {
                        end_command_semicolon = false;
                    } else {
                        error_str("Missing a semicolon after variable declaration", pos);
                    }
                    var_static_set = false;
                    parsed = true;
                } else if var_dynamic_set {
                    let mut require_semicolon = true;
                    if let Some(tok) = list.pop() {
                        if let TokenType::Raw(name) = &tok.token {
                            debug(format!("change variable {} to {}", name, instr.clone()));
                            list.push(Token {
                                token: TokenType::VariableDynamicSet(name.clone(), instr.clone(), var_cast),
                                pos
                            });
                            var_dynamic_set = false;
                        } else if let TokenType::InitVariable(name, typ, _, _, value) = &tok.token {
                            if value.is_some() {
                                error(format!(r#"Can't initiate variable "{}" twice!"#, name), pos);
                            }
                            debug(format!("initiate variable {} of type {} to {}", name, typ, instr.clone()));
                            list.push(Token {
                                token: TokenType::InitVariable(name.clone(), typ.clone(), false, var_cast, Some(instr.clone())),
                                pos
                            });
                            var_dynamic_set = false;
                        } else if let TokenType::CallEvent(name, _) = &tok.token {
                            event_params.push(instr.clone());
                            if end_command_semicolon {
                                list.push(Token {
                                    token: TokenType::CallEvent(name.clone(), Some(event_params)),
                                    pos
                                });
                                event_params = vec![];
                                var_dynamic_set = false;
                            } else {
                                list.push(tok);
                                require_semicolon = false;
                            }
                        } else if let TokenType::InitVariableEvent(event, name, _) = &tok.token {
                            event_params.push(instr.clone());
                            if end_command_semicolon {
                                list.push(Token {
                                    token: TokenType::InitVariableEvent(event.clone(), name.clone(), Some(event_params)),
                                    pos
                                });
                                event_params = vec![];
                                var_dynamic_set = false;
                            } else {
                                list.push(tok);
                                require_semicolon = false;
                            }
                        } else if let TokenType::VariableEventSet(name, event, _) = &tok.token {
                            event_params.push(instr.clone());
                            if end_command_semicolon {
                                list.push(Token {
                                    token: TokenType::VariableEventSet(name.clone(), event.clone(), Some(event_params)),
                                    pos
                                });
                                event_params = vec![];
                                var_dynamic_set = false;
                            } else {
                                list.push(tok);
                                require_semicolon = false;
                            }
                        } else {
                            error_str("No variable name specified", pos);
                        }
                    }
                    if end_command_semicolon {
                        end_command_semicolon = false;
                    } else if require_semicolon {
                        error_str("Missing a semicolon after dynamic declaration", pos);
                    }
                    parsed = true;
                }
                if !parsed {
                    list.push(Token {
                        token: TokenType::Raw(instr.clone()),
                        pos
                    });
                    parsed = true;
                }
            }
        } else {
            list.push(Token {
                token: TokenType::Listener(instr.clone()),
                pos
            });
            listener_type = Some(instr.clone());
            debug(format!("entered listener {}", instr));
            if instr.ends_with('{') {
                warning(format!(r#"Weird Event Name "{}", did you forget a whitespace?"#, instr), pos);
            }
            parsed = true;
        }

        if end_command_semicolon {
            error(format!("Unexpected Semicolon"), pos);
        }

        if !parsed {
            error(format!("Unexpected Token: {}", instr), pos);
        }

        (scope_depth, var_static_set, var_dynamic_set, var_cast, listener_type, event_params, list)
    });

    if scope_depth > 0 {
        error_unknown_pos(format!("Missing exit for {} scope(s)", scope_depth));
    }

    list
}

pub fn split(tokens: Vec<Token>) -> HashMap<String, Vec<EventListener>> {
    let mut map: HashMap<String, Vec<EventListener>> = HashMap::new();
    tokens.iter().fold((0 as u8, None, vec![]), |acc: (u8, Option<String>, Vec<Token>), token| {
        let (mut scope_depth, mut event_type, mut list) = acc;

        if let Some(typ) = &event_type {
            if let TokenType::ScopeStart = &token.token {
                scope_depth += 1;
            } else if let TokenType::ScopeEnd = &token.token {
                scope_depth -= 1;
            }
            list.push(token.clone());

            if scope_depth == 0 {
                if let Some(listeners) = map.get_mut(typ) {
                    listeners.push(EventListener {
                        tokens: list
                    });
                } else {
                    map.insert(typ.clone(), vec![EventListener {
                        tokens: list
                    }]);
                }
                event_type = None;
                list = vec![];
            }
        } else if let TokenType::Listener(typ) = &token.token {
            event_type = Some(typ.clone());
        }

        (scope_depth, event_type.clone(), list)
    });

    map
}