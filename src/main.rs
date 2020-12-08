use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result};
use termion::{color, style};

mod dynfmt;
mod view;

use view::Choice;

#[derive(Debug)]
struct Command {
    cmd: String,
    description: String,
    flags: Vec<Flag>,
    arguments: Vec<Arg>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct Flag {
    template: String,
    description: String,
    expected_value: Option<ValueType>,
}

#[derive(Debug)]
struct Arg {
    description: String,
    expected_value: Option<ValueType>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum ValueType {
    String,
    Path,
    Number,
}

fn main() {
    match build_cmd() {
        Ok(Some(cmd)) => {
            println!("Done: {}", cmd);
        }
        Ok(None) => {
            println!("Nothing selected.");
        }
        Err(err) => {
            eprintln!("Failed: {:?}", err);
        }
    }
}

/// Build command and return the result
fn build_cmd() -> Result<Option<String>> {
    let commands = vec![
        Command {
            // TODO: couldn't this be a template also?
            cmd: "find".into(),
            description: "Find file or directory".into(),
            flags: vec![Flag {
                template: "-name {name}".into(),
                description: "File name (glob syntax)".into(),
                expected_value: Some(ValueType::String),
            }],
            arguments: vec![Arg {
                description: "Directory to search in".into(),
                expected_value: Some(ValueType::Path),
            }],
        },
        Command {
            cmd: "grep".into(),
            description: "Find lines in a file".into(),
            flags: vec![
                Flag {
                    template: "-i".into(),
                    description: "Case insensitive matching".into(),
                    expected_value: None,
                },
                Flag {
                    template: "-A{num}".into(),
                    description: "Print {num} lines after the matched line".into(),
                    expected_value: Some(ValueType::Number),
                },
            ],
            arguments: vec![
                Arg {
                    description: "Pattern".into(),
                    expected_value: Some(ValueType::String),
                },
                Arg {
                    description: "File".into(),
                    expected_value: Some(ValueType::Path),
                },
            ],
        },
    ];

    // TODO: steps:
    // 1. Type to filter commands by description
    // 2. Type to filter command options by description
    // 3. If option expects a value than input it
    // 4. Present command and exit

    let mut result = String::new();
    println!("Choose command:");
    // TODO: in here I want a fixed value and the command not just input!
    // Readline should return a value from autocomplete method!
    let cmd = view::Readline::new()
        .autocomplete(|input| {
            commands
                .iter()
                .filter(|cmd| cmd.description.contains(input))
                .collect()
        })
        .context("Pick command")?;

    let cmd = match cmd {
        Some(c) => c,
        None => {
            return Ok(None);
        }
    };

    result.push_str(&cmd.cmd);
    println!("Command: {}", result);

    println!("Choose flags: (choose empty to skip)");
    // Prompt for flags
    let mut available_flags: HashSet<&Flag> = cmd.flags.iter().collect();

    loop {
        if available_flags.is_empty() {
            break;
        }

        // TODO: filter out already selected flags
        // TODO: skip this step if there are no more flags
        let flag = view::Readline::new()
            .autocomplete(|input| {
                available_flags
                    .iter()
                    .filter(|flag| flag.description.contains(input))
                    .cloned()
                    .collect()
            })
            .context("Pick flag")?;

        let flag = match flag {
            Some(flag) => flag,
            None => {
                break;
            }
        };

        available_flags.remove(flag);

        // Enter value
        // TODO: ideally I want to split the template into to parts: prefix and suffix so that we can actually show building command
        let value = match flag.expected_value {
            Some(ValueType::Path) => view::Readline::new().prefix(&result).line()?,
            Some(ValueType::String) => view::Readline::new().prefix(&result).line()?,
            Some(ValueType::Number) => view::Readline::new().prefix(&result).line()?,
            None => {
                result.push(' ');
                result.push_str(&flag.template);
                println!("Command: {}", result);
                continue;
            }
        };

        result.push(' ');
        let mut ctx = HashMap::new();
        // TODO: if value is not only alphanumeric then wrap inside quotes
        ctx.insert("name".to_string(), value);
        let applied = dynfmt::format(&flag.template, ctx)?;
        result.push_str(&applied);
        println!("Command: {}", result);
    }

    // Prompt for commands
    for arg in &cmd.arguments {
        let value = match arg.expected_value {
            Some(ValueType::Path) => view::Readline::new().prefix(&arg.description).line()?,
            Some(ValueType::String) => view::Readline::new().prefix(&arg.description).line()?,
            Some(ValueType::Number) => view::Readline::new().prefix(&arg.description).line()?,
            None => {
                continue;
            }
        };

        result.push(' ');
        result.push_str(&value);
    }

    Ok(Some(result))
}

impl Choice for Command {
    type Text = String;

    fn text(&self) -> &Self::Text {
        &self.description
    }
}

impl Choice for Flag {
    type Text = String;

    fn text(&self) -> &Self::Text {
        &self.description
    }
}
