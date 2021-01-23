use std::io::Write;
use std::{collections::HashMap, io::stdout};

use anyhow::{anyhow, Context, Result};
use termion::{raw::IntoRawMode, screen::AlternateScreen};

mod builtin;
mod cmd;
// TODO: do we still need it?
// mod dynfmt;
mod view;

use cmd::*;
use view::Choice;

fn main() {
    match build_cmd() {
        // match play() {
        Ok(Some(cmd)) => {
            println!("{}", cmd);
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
    let commands = builtin::all();
    let mut screen = AlternateScreen::from(stdout().into_raw_mode()?);

    // TODO: show a list of cmd descriptions and a name of the command
    // E.g: - Find lines in a file (find)
    // E.g: - Find files or directories (grep)
    let cmd = view::Readline::new(&mut screen)?
        .help("Pick a command:")
        .autocomplete(|input| {
            commands
                .iter()
                .filter(|cmd| {
                    cmd.description
                        .to_lowercase()
                        .contains(&input.to_lowercase())
                })
                .collect()
        })
        .context("Pick command")?;

    let cmd = match cmd {
        Some(c) => c,
        None => {
            return Ok(None);
        }
    };

    writeln!(&mut screen, "{}\n", cmd.template)?;
    let mut user_input = HashMap::new();

    for group in &cmd.groups {
        match &group.expect {
            GroupValue::Single(expect_type) => {
                let prefix = format!("{}:", group.name);
                let value = view::Readline::new(&mut screen)?
                    .prefix(&prefix)
                    .expect(expect_type.clone())
                    .line()?;
                if value.is_empty() {
                    return Err(anyhow!("No value for {} group", group.name));
                }
                user_input.insert(group.name.clone(), value);
                let result = (cmd.build)(&user_input);
                writeln!(&mut screen, "{}\n", result)?;
            }
            GroupValue::Flags(flags) => {
                let mut used_flags = vec![];
                let mut combined = String::new();
                user_input.insert(group.name.clone(), combined.clone());

                loop {
                    let flag = view::Readline::new(&mut screen)?
                        .help((cmd.build)(&user_input))
                        .autocomplete(|input| {
                            flags
                                .iter()
                                .filter(|flag| !used_flags.contains(flag))
                                .filter(|flag| {
                                    flag.description
                                        .to_lowercase()
                                        .contains(&input.to_lowercase())
                                })
                                .collect()
                        })
                        .context("Pick a flag")?;

                    match flag {
                        Some(flag) => {
                            // Remember that this flag was asked
                            // TODO: allow setting multiple same flags (e.g headers in curl)
                            used_flags.push(flag);

                            match &flag.expect {
                                // Ask for input
                                Some(expect) => match expect.value_type {
                                    ValueType::String | ValueType::Path | ValueType::Number => {
                                        let prefix = format!("{}:", flag.template);
                                        let value = view::Readline::new(&mut screen)?
                                            .prefix(&prefix)
                                            .help(&flag.description)
                                            .expect(expect.value_type.clone())
                                            .line()?;
                                        if value.is_empty() {
                                            return Err(anyhow!(
                                                "No value for {} flag",
                                                flag.template
                                            ));
                                        }
                                        let result = (expect.build)(&value);
                                        combined.push_str(&result);
                                        combined.push(' ');
                                    }
                                },
                                // Save flag
                                None => {
                                    combined.push_str(&flag.template);
                                    combined.push(' ');
                                }
                            }
                        }
                        None => {
                            // Nothing selected abort
                            break;
                        }
                    }

                    user_input.insert(group.name.clone(), combined.clone());

                    if flags.len() == used_flags.len() {
                        break;
                    }
                }
            }
        }
    }

    let result = (cmd.build)(&user_input);
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
