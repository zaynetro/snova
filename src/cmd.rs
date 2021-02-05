use std::collections::HashMap;

use anyhow::{Result, anyhow};

pub struct Command {
    pub template: String,
    pub description: String,
    pub groups: Vec<CmdGroup>,
    pub build: Box<dyn Fn(&HashMap<String, String>) -> String>,
}

pub struct CmdGroup {
    pub name: String,
    pub expect: GroupValue,
    pub optional: bool,
}

pub enum GroupValue {
    Single(ValueType),
    Flags(Vec<Flag>),
}

pub struct Flag {
    pub template: String,
    pub description: String,
    pub expect: Option<FlagExpectation>,
    /// Allow specifing this flag multiple times
    pub multiple: bool,
    pub suggest: Option<Vec<String>>,
}

impl PartialEq for Flag {
    fn eq(&self, other: &Self) -> bool {
        self.template == other.template && self.description == other.description
    }
}

pub struct FlagExpectation {
    pub build: Box<dyn Fn(&str) -> String>,
    pub value_type: ValueType,
}

#[derive(Debug, Clone)]
// TODO: support enum value type (e.g request method in curl: GET/POST/...)
pub enum ValueType {
    String,
    Path,
    Number,
}

impl ValueType {
    pub fn is_valid_char(&self, c: char) -> bool {
        match self {
            ValueType::String | ValueType::Path => true,
            ValueType::Number => c.is_digit(10),
        }
    }

    pub fn parse(v: &str) -> Result<ValueType> {
        match v {
            "string" => Ok(ValueType::String),
            "path" => Ok(ValueType::Path),
            "number" => Ok(ValueType::Number),
            _ => Err(anyhow!("Unknown value type '{}'", v))
        }
    }
}
