use std::collections::HashMap;

pub struct Command {
    pub template: String,
    pub description: String,
    pub groups: Vec<CmdGroup>,
    pub build: Box<dyn Fn(&HashMap<String, String>) -> String>,
}

pub struct CmdGroup {
    pub name: String,
    pub expect: GroupValue,
}

pub enum GroupValue {
    Single(ValueType),
    Flags(Vec<Flag>),
}

pub struct Flag {
    pub template: String,
    pub description: String,
    pub expect: Option<FlagExpectation>,
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
}