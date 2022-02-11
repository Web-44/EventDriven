use std::any::Any;
use std::str::FromStr;
use substring::Substring;
use crate::event::standard_events::Event;
use crate::util::{error, error_str, runtime_error, runtime_error_unknown_pos};
use crate::util::position::Position;

#[derive(Debug, Clone)]
pub enum VariableType {
    U8(Option<u8>),
    U16(Option<u16>),
    U32(Option<u32>),
    U64(Option<u64>),
    I8(Option<i8>),
    I16(Option<i16>),
    I32(Option<i32>),
    I64(Option<i64>),
    I128(Option<i128>),
    Char(Option<char>),
    String(Option<String>),
    Bool(Option<bool>),
    Float(Option<f32>),
    Double(Option<f64>),
    Event(Box<dyn Event>)
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub variable_type: VariableType,
    pub nullable: bool
}

macro_rules! var_type {
    ($value:expr,$pos:expr,$typ_literal:expr,$typ:ty) => {{
        if let Ok(value) = <$typ>::from_str($value.as_str()) {
            Some(value)
        } else {
            error(format!(r#"{} is not of type {}"#, $value, $typ_literal), $pos);
            unreachable!();
        }
    }}
}

impl Variable {
    pub fn new(name: &String, typ: &String, value: &String, pos: Position) -> Self {
        let mut typ = typ.clone();
        let nullable = typ.starts_with('?');
        if nullable {
            typ.remove(0);
        }
        let set_null = value == "null";
        if set_null && !nullable {
            runtime_error(format!("Can't assign null to non-null variable {}", name), pos);
        }
        let variable_type = if set_null {
            name_to_type_null(&typ, pos)
        } else {
            name_to_type(&typ, value, pos)
        };

        Variable {
            name: name.clone(),
            variable_type,
            nullable
        }
    }

    pub fn new_event_type(name: &String, event: Box<dyn Event>) -> Self {
        Variable {
            name: name.clone(),
            variable_type: VariableType::Event(event),
            nullable: false
        }
    }

    pub fn new_type(name: &String, typ: &String, cast: bool, value: VariableType, pos: Position) -> Self {
        let mut typ = typ.clone();
        let nullable = typ.starts_with('?');
        if nullable {
            typ.remove(0);
        }
        let string_value = type_value_to_string(&value);
        let set_null = string_value.is_none();
        if set_null && !nullable {
            runtime_error(format!("Can't assign null to non-null variable {}", name), pos);
        }
        let variable_type = name_to_type_null(&typ, pos);
        if check_same_type(&variable_type, &value) {
            return Variable {
                name: name.clone(),
                variable_type: value,
                nullable
            };
        } else if cast {
            let variable_type = if set_null {
                name_to_type_null(&typ, pos)
            } else {
                name_to_type(&typ, &string_value.unwrap(), pos)
            };
            return Variable {
                name: name.clone(),
                variable_type,
                nullable
            };
        } else {
            runtime_error(format!("Can't assign type {} to variable {} of type {}", type_to_name(value), name, typ), pos);
            unreachable!();
        }
    }

    pub fn copy(var: Variable, value: &String, pos: Position) -> Self {
        let set_null = value == "null";
        if set_null && !var.nullable {
            runtime_error(format!("Can't assign null to non-null variable {}", var.name), pos);
        }

        let variable_type = if set_null {
            clone_type_null(var.variable_type, pos)
        } else {
            clone_type(var.variable_type, value, pos)
        };

        Variable {
            name: var.name,
            variable_type,
            nullable: var.nullable
        }
    }

    pub fn copy_type(var: Variable, cast: bool, value: VariableType, pos: Position) -> Self {
        let string_value = type_value_to_string(&value);
        let set_null = string_value.is_none();
        if set_null && !var.nullable {
            runtime_error(format!("Can't assign null to non-null variable {}", var.name), pos);
        }
        if var.variable_type.type_id() == value.type_id() {
            return Variable {
                name: var.name.clone(),
                variable_type: value,
                nullable: var.nullable
            };
        } else if cast {
            let variable_type = if set_null {
                name_to_type_null(&type_to_name(var.variable_type).to_string(), pos)
            } else {
                name_to_type(&type_to_name(var.variable_type).to_string(), &string_value.unwrap(), pos)
            };
            return Variable {
                name: var.name.clone(),
                variable_type,
                nullable: var.nullable
            };
        } else {
            runtime_error(format!("Can't assign type {} to variable {} of type {}", type_to_name(value), var.name, type_to_name(var.variable_type)), pos);
            unreachable!();
        }
    }
}

fn type_to_name(typ: VariableType) -> &'static str {
    match typ {
        VariableType::U8(_) => "u8",
        VariableType::U16(_) => "u16",
        VariableType::U32(_) => "u32",
        VariableType::U64(_) => "u64",
        VariableType::I8(_) => "i8",
        VariableType::I16(_) => "i16",
        VariableType::I32(_) => "i32",
        VariableType::I64(_) => "i64",
        VariableType::I128(_) => "i128",
        VariableType::Char(_) => "char",
        VariableType::String(_) => "string",
        VariableType::Bool(_) => "bool",
        VariableType::Float(_) => "float",
        VariableType::Double(_) => "double",
        VariableType::Event(_) => "event"
    }
}

pub fn type_value_to_number(value: &VariableType) -> Option<i128> {
    match value {
        VariableType::U8(val) => if let Some(val) = val { Some(*val as i128) } else { None },
        VariableType::U16(val) => if let Some(val) = val { Some(*val as i128) } else { None },
        VariableType::U32(val) => if let Some(val) = val { Some(*val as i128) } else { None },
        VariableType::U64(val) => if let Some(val) = val { Some(*val as i128) } else { None },
        VariableType::I8(val) => if let Some(val) = val { Some(*val as i128) } else { None },
        VariableType::I16(val) => if let Some(val) = val { Some(*val as i128) } else { None },
        VariableType::I32(val) => if let Some(val) = val { Some(*val as i128) } else { None },
        VariableType::I64(val) => if let Some(val) = val { Some(*val as i128) } else { None },
        VariableType::I128(val) => if let Some(val) = val { Some(*val) } else { None },
        _ => {
            runtime_error_unknown_pos(format!("Can't get variable type {:?} as a number", value));
            unreachable!();
        }
    }
}

pub fn type_value_to_string(value: &VariableType) -> Option<String> {
    match value {
        VariableType::U8(val) => if let Some(val) = val { Some(format!("{}", val)) } else { None },
        VariableType::U16(val) => if let Some(val) = val { Some(format!("{}", val)) } else { None },
        VariableType::U32(val) => if let Some(val) = val { Some(format!("{}", val)) } else { None },
        VariableType::U64(val) => if let Some(val) = val { Some(format!("{}", val)) } else { None },
        VariableType::I8(val) => if let Some(val) = val { Some(format!("{}", val)) } else { None },
        VariableType::I16(val) => if let Some(val) = val { Some(format!("{}", val)) } else { None },
        VariableType::I32(val) => if let Some(val) = val { Some(format!("{}", val)) } else { None },
        VariableType::I64(val) => if let Some(val) = val { Some(format!("{}", val)) } else { None },
        VariableType::I128(val) => if let Some(val) = val { Some(format!("{}", val)) } else { None },
        VariableType::Char(val) => if let Some(val) = val { Some(String::from(val.clone())) } else { None },
        VariableType::String(val) => if let Some(val) = val { Some(val.clone()) } else { None },
        VariableType::Bool(val) => if let Some(val) = val { Some(format!("{}", val)) } else { None }
        VariableType::Float(val) => if let Some(val) = val { Some(format!("{}", val)) } else { None },
        VariableType::Double(val) => if let Some(val) = val { Some(format!("{}", val)) } else { None },
        VariableType::Event(val) => Some(val.name().to_string())
    }
}

fn name_to_type(name: &String, value: &String, pos: Position) -> VariableType {
    match name.as_str() {
        "u8" => VariableType::U8(var_type!(value,pos,name,u8)),
        "u16" => VariableType::U16(var_type!(value,pos,name,u16)),
        "u32" => VariableType::U32(var_type!(value,pos,name,u32)),
        "u64" => VariableType::U64(var_type!(value,pos,name,u64)),
        "i8" => VariableType::I8(var_type!(value,pos,name,i8)),
        "i16" => VariableType::I16(var_type!(value,pos,name,i16)),
        "i32" => VariableType::I32(var_type!(value,pos,name,i32)),
        "i64" => VariableType::I64(var_type!(value,pos,name,i64)),
        "i128" => {
            error_str("Can't directly initialize i128 type", pos);
            unreachable!()
        },
        "char" => VariableType::Char(var_type!(value,pos,name,char)),
        "string" => {
            if value.starts_with('"') && value.ends_with('"') {
                VariableType::String(Some(value.substring(1, value.len() - 1).to_string()))
            } else {
                error(format!(r#"{} is not of type string"#, value), pos);
                unreachable!()
            }
        },
        "bool" => VariableType::Bool(var_type!(value,pos,name,bool)),
        "float" => VariableType::Float(var_type!(value,pos,name,f32)),
        "double" => VariableType::Double(var_type!(value,pos,name,f64)),
        "event" => {
            error_str("Can't directly initialize event type", pos);
            unreachable!()
        }
        _ => {
            error(format!("Invalid Variable Type: {}", name), pos);
            unreachable!()
        }
    }
}

fn name_to_type_null(name: &String, pos: Position) -> VariableType {
    match name.as_str() {
        "u8" => VariableType::U8(None),
        "u16" => VariableType::U16(None),
        "u32" => VariableType::U32(None),
        "u64" => VariableType::U64(None),
        "i8" => VariableType::I8(None),
        "i16" => VariableType::I16(None),
        "i32" => VariableType::I32(None),
        "i64" => VariableType::I64(None),
        "i128" => {
            error_str("Can't directly initialize i128 type", pos);
            unreachable!()
        },
        "char" => VariableType::Char(None),
        "string" => VariableType::String(None),
        "bool" => VariableType::Bool(None),
        "float" => VariableType::Float(None),
        "double" => VariableType::Double(None),
        "event" => {
            error_str("Can't directly initialize event type", pos);
            unreachable!();
        }
        _ => {
            error(format!("Invalid Variable Type: {}", name), pos);
            unreachable!()
        }
    }
}

fn clone_type(typ: VariableType, value: &String, pos: Position) -> VariableType {
    match typ {
        VariableType::U8(_) => VariableType::U8(var_type!(value,pos,"u8",u8)),
        VariableType::U16(_) => VariableType::U16(var_type!(value,pos,"u16",u16)),
        VariableType::U32(_) => VariableType::U32(var_type!(value,pos,"u32",u32)),
        VariableType::U64(_) => VariableType::U64(var_type!(value,pos,"u64",u64)),
        VariableType::I8(_) => VariableType::I8(var_type!(value,pos,"i8",i8)),
        VariableType::I16(_) => VariableType::I16(var_type!(value,pos,"i16",i16)),
        VariableType::I32(_) => VariableType::I32(var_type!(value,pos,"i32",i32)),
        VariableType::I64(_) => VariableType::I64(var_type!(value,pos,"i64",i64)),
        VariableType::I128(_) => {
            error_str("Can't directly initialize i128 type", pos);
            unreachable!()
        },
        VariableType::Char(_) => VariableType::Char(var_type!(value,pos,"char",char)),
        VariableType::String(_) => {
            if value.starts_with('"') && value.ends_with('"') {
                VariableType::String(Some(value.substring(1, value.len() - 1).to_string()))
            } else {
                error(format!(r#"{} is not of type string"#, value), pos);
                unreachable!()
            }
        },
        VariableType::Bool(_) => VariableType::Bool(var_type!(value,pos,"bool",bool)),
        VariableType::Float(_) => VariableType::Float(var_type!(value,pos,"float",f32)),
        VariableType::Double(_) => VariableType::Double(var_type!(value,pos,"double",f64)),
        VariableType::Event(_) => {
            error_str("Can't directly initialize event type", pos);
            unreachable!();
        }
    }
}

fn clone_type_null(typ: VariableType, pos: Position) -> VariableType {
    match typ {
        VariableType::U8(_) => VariableType::U8(None),
        VariableType::U16(_) => VariableType::U16(None),
        VariableType::U32(_) => VariableType::U32(None),
        VariableType::U64(_) => VariableType::U64(None),
        VariableType::I8(_) => VariableType::I8(None),
        VariableType::I16(_) => VariableType::I16(None),
        VariableType::I32(_) => VariableType::I32(None),
        VariableType::I64(_) => VariableType::I64(None),
        VariableType::I128(_) => {
            error_str("Can't directly initialize event type", pos);
            unreachable!();
        }
        VariableType::Char(_) => VariableType::Char(None),
        VariableType::String(_) => VariableType::String(None),
        VariableType::Bool(_) => VariableType::Bool(None),
        VariableType::Float(_) => VariableType::Float(None),
        VariableType::Double(_) => VariableType::Double(None),
        VariableType::Event(_) => {
            error_str("Can't directly initialize event type", pos);
            unreachable!();
        }
    }
}

fn check_same_type(typ1: &VariableType, typ2: &VariableType) -> bool {
    match (typ1, typ2) {
        (VariableType::U8(_), VariableType::U8(_)) => true,
        (VariableType::U16(_), VariableType::U16(_)) => true,
        (VariableType::U32(_), VariableType::U32(_)) => true,
        (VariableType::U64(_), VariableType::U64(_)) => true,
        (VariableType::I8(_), VariableType::I8(_)) => true,
        (VariableType::I16(_), VariableType::I16(_)) => true,
        (VariableType::I32(_), VariableType::I32(_)) => true,
        (VariableType::I64(_), VariableType::I64(_)) => true,
        (VariableType::Char(_), VariableType::Char(_)) => true,
        (VariableType::String(_), VariableType::String(_)) => true,
        (VariableType::Bool(_), VariableType::Bool(_)) => true,
        (VariableType::Float(_), VariableType::Float(_)) => true,
        (VariableType::Double(_), VariableType::Double(_)) => true,
        (VariableType::Event(_), VariableType::Event(_)) => true,
        _ => false
    }
}