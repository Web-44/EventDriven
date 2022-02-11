use std::fmt::{Debug, Formatter};
use crate::util::debug::debug_str;
use crate::util::variables::{type_value_to_number, type_value_to_string, VariableType};

pub fn get_event(name: &String) -> Option<Box<dyn Event>> {
    match name.as_str() {
        "OnStart" => Some(Box::new(OnStart)),
        "Print" => Some(Box::new(Print::default())),
        "+" => Some(Box::new(MathAdd::default())),
        "-" => Some(Box::new(MathSubtract::default())),
        "*" => Some(Box::new(MathMultiply::default())),
        "/" => Some(Box::new(MathDivide::default())),
        "%" => Some(Box::new(MathModulo::default())),
        _ => None
    }
}

impl Debug for dyn Event {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.name())
    }
}

impl Clone for Box<dyn Event> {
    fn clone(&self) -> Self {
        self.clone_self()
    }
}

pub trait Event {
    fn name(&self) -> &str;
    fn check_param_count(&self, count: u8) -> bool;
    fn get_var(&self, name: &String) -> Option<VariableType>;
    fn accept(&mut self, idx: u8, param: VariableType) -> bool;
    fn call(&mut self);
    fn clone_self(&self) -> Box<dyn Event>;
}

pub struct OnStart;
impl Event for OnStart {
    fn name(&self) -> &str {
        "OnStart"
    }
    fn check_param_count(&self, count: u8) -> bool {
        count == 0
    }
    fn accept(&mut self, _idx: u8, _param: VariableType) -> bool {
        false
    }
    fn get_var(&self, _name: &String) -> Option<VariableType> {
        None
    }
    fn call(&mut self) {
        debug_str("START EVENT CALLED");
    }
    fn clone_self(&self) -> Box<dyn Event> {
        Box::new(OnStart)
    }
}

#[derive(Default)]
pub struct Print {
    message: Option<String>
}
impl Event for Print {
    fn name(&self) -> &str {
        "Print"
    }
    fn check_param_count(&self, count: u8) -> bool {
        count > 0
    }
    fn accept(&mut self, _idx: u8, param: VariableType) -> bool {
        if let Some(message) = &mut self.message {
            if let Some(add) = type_value_to_string(&param) {
                message.push_str(add.as_str());
            } else {
                message.push_str("null");
            }
        } else {
            self.message = type_value_to_string(&param);
        }
        true
    }
    fn get_var(&self, name: &String) -> Option<VariableType> {
        match name.as_str() {
            "message" => Some(VariableType::String(self.message.clone())),
            _ => None
        }
    }
    fn call(&mut self) {
        if let Some(message) = &self.message {
            println!("{}", message);
        } else {
            println!("null");
        }
    }
    fn clone_self(&self) -> Box<dyn Event> {
        Box::new(Print {
            message: self.message.clone()
        })
    }
}

#[derive(Default)]
pub struct MathAdd {
    num1: Option<i128>,
    num2: Option<i128>,
    result: Option<i128>
}
impl Event for MathAdd {
    fn name(&self) -> &str {
        "+"
    }
    fn check_param_count(&self, count: u8) -> bool {
        count == 2
    }
    fn get_var(&self, name: &String) -> Option<VariableType> {
        match name.as_str() {
            "num1" => Some(VariableType::I128(self.num1.clone())),
            "num2" => Some(VariableType::I128(self.num2.clone())),
            "result" => Some(VariableType::I128(self.result.clone())),
            _ => None
        }
    }
    fn accept(&mut self, idx: u8, param: VariableType) -> bool {
        match idx {
            0 => {
                self.num1 = type_value_to_number(&param);
                true
            },
            1 => {
                self.num2 = type_value_to_number(&param);
                true
            },
            _ => false
        }
    }
    fn call(&mut self) {
        if let Some(num1) = &self.num1 {
            if let Some(num2) = &self.num2 {
                self.result = Some(num1 + num2);
            }
        }
    }
    fn clone_self(&self) -> Box<dyn Event> {
        Box::new(MathAdd {
            num1: self.num1,
            num2: self.num2,
            result: self.result
        })
    }
}

#[derive(Default)]
pub struct MathSubtract {
    num1: Option<i128>,
    num2: Option<i128>,
    result: Option<i128>
}
impl Event for MathSubtract {
    fn name(&self) -> &str {
        "-"
    }
    fn check_param_count(&self, count: u8) -> bool {
        count == 2
    }
    fn get_var(&self, name: &String) -> Option<VariableType> {
        match name.as_str() {
            "num1" => Some(VariableType::I128(self.num1.clone())),
            "num2" => Some(VariableType::I128(self.num2.clone())),
            "result" => Some(VariableType::I128(self.result.clone())),
            _ => None
        }
    }
    fn accept(&mut self, idx: u8, param: VariableType) -> bool {
        match idx {
            0 => {
                self.num1 = type_value_to_number(&param);
                true
            },
            1 => {
                self.num2 = type_value_to_number(&param);
                true
            },
            _ => false
        }
    }
    fn call(&mut self) {
        if let Some(num1) = &self.num1 {
            if let Some(num2) = &self.num2 {
                self.result = Some(num1 - num2);
            }
        }
    }
    fn clone_self(&self) -> Box<dyn Event> {
        Box::new(MathAdd {
            num1: self.num1,
            num2: self.num2,
            result: self.result
        })
    }
}

#[derive(Default)]
pub struct MathMultiply {
    num1: Option<i128>,
    num2: Option<i128>,
    result: Option<i128>
}
impl Event for MathMultiply {
    fn name(&self) -> &str {
        "*"
    }
    fn check_param_count(&self, count: u8) -> bool {
        count == 2
    }
    fn get_var(&self, name: &String) -> Option<VariableType> {
        match name.as_str() {
            "num1" => Some(VariableType::I128(self.num1.clone())),
            "num2" => Some(VariableType::I128(self.num2.clone())),
            "result" => Some(VariableType::I128(self.result.clone())),
            _ => None
        }
    }
    fn accept(&mut self, idx: u8, param: VariableType) -> bool {
        match idx {
            0 => {
                self.num1 = type_value_to_number(&param);
                true
            },
            1 => {
                self.num2 = type_value_to_number(&param);
                true
            },
            _ => false
        }
    }
    fn call(&mut self) {
        if let Some(num1) = &self.num1 {
            if let Some(num2) = &self.num2 {
                self.result = Some(num1 * num2);
            }
        }
    }
    fn clone_self(&self) -> Box<dyn Event> {
        Box::new(MathAdd {
            num1: self.num1,
            num2: self.num2,
            result: self.result
        })
    }
}

#[derive(Default)]
pub struct MathDivide {
    num1: Option<i128>,
    num2: Option<i128>,
    result: Option<i128>
}
impl Event for MathDivide {
    fn name(&self) -> &str {
        "/"
    }
    fn check_param_count(&self, count: u8) -> bool {
        count == 2
    }
    fn get_var(&self, name: &String) -> Option<VariableType> {
        match name.as_str() {
            "num1" => Some(VariableType::I128(self.num1.clone())),
            "num2" => Some(VariableType::I128(self.num2.clone())),
            "result" => Some(VariableType::I128(self.result.clone())),
            _ => None
        }
    }
    fn accept(&mut self, idx: u8, param: VariableType) -> bool {
        match idx {
            0 => {
                self.num1 = type_value_to_number(&param);
                true
            },
            1 => {
                self.num2 = type_value_to_number(&param);
                true
            },
            _ => false
        }
    }
    fn call(&mut self) {
        if let Some(num1) = &self.num1 {
            if let Some(num2) = &self.num2 {
                self.result = Some(num1 / num2);
            }
        }
    }
    fn clone_self(&self) -> Box<dyn Event> {
        Box::new(MathAdd {
            num1: self.num1,
            num2: self.num2,
            result: self.result
        })
    }
}

#[derive(Default)]
pub struct MathModulo {
    num1: Option<i128>,
    num2: Option<i128>,
    result: Option<i128>
}
impl Event for MathModulo {
    fn name(&self) -> &str {
        "%"
    }
    fn check_param_count(&self, count: u8) -> bool {
        count == 2
    }
    fn get_var(&self, name: &String) -> Option<VariableType> {
        match name.as_str() {
            "num1" => Some(VariableType::I128(self.num1.clone())),
            "num2" => Some(VariableType::I128(self.num2.clone())),
            "result" => Some(VariableType::I128(self.result.clone())),
            _ => None
        }
    }
    fn accept(&mut self, idx: u8, param: VariableType) -> bool {
        match idx {
            0 => {
                self.num1 = type_value_to_number(&param);
                true
            },
            1 => {
                self.num2 = type_value_to_number(&param);
                true
            },
            _ => false
        }
    }
    fn call(&mut self) {
        if let Some(num1) = &self.num1 {
            if let Some(num2) = &self.num2 {
                self.result = Some(num1 % num2);
            }
        }
    }
    fn clone_self(&self) -> Box<dyn Event> {
        Box::new(MathAdd {
            num1: self.num1,
            num2: self.num2,
            result: self.result
        })
    }
}