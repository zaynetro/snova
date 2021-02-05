//! Parses commands definition files

use std::collections::{HashMap, VecDeque};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use crate::cmd::*;

/// Builtin commands
const BUILTIN_DEF: &'static str = include_str!("../defs/builtin.toml");

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandsDef {
    commands: VecDeque<CommandDef>,
}

/// A single command definition in the config file
#[derive(Debug, Serialize, Deserialize)]
struct CommandDef {
    template: String,
    description: String,
    groups: HashMap<String, GroupDef>,
}

/// A single group definition in the config file
#[derive(Debug, Serialize, Deserialize)]
struct GroupDef {
    expect: Option<ValueTypeDef>,
    flags: Option<VecDeque<FlagDef>>,
}

type ValueTypeDef = String;

#[derive(Debug, Serialize, Deserialize)]
struct FlagDef {
    template: String,
    description: String,
    expect: Option<ValueTypeDef>,
    #[serde(default)]
    multiple: bool,
    suggest: Option<Vec<String>>,
}

/// Read all commands
pub fn read_all() -> Result<Vec<Command>> {
    let mut all = builtin()?;
    let mut user = user_commands()?;
    all.append(&mut user);
    Ok(all)
}

/// Read user commands
fn user_commands() -> Result<Vec<Command>> {
    if let Some(mut config_dir) = dirs::config_dir() {
        config_dir.push("snova");
        if config_dir.is_dir() {
            let commands_file = config_dir.join("commands.toml");
            if commands_file.is_file() {
                // Try reading user commands file
                let data = std::fs::read_to_string(&commands_file)
                    .context(format!("Read {}", commands_file.display()))?;
                let defs: CommandsDef =
                    toml::de::from_str(&data).context("Parse user commands toml")?;
                return parse_defs(defs)
            }
        }
    }

    Ok(vec![])
}

/// Read builtin commands
fn builtin() -> Result<Vec<Command>> {
    let defs: CommandsDef =
        toml::de::from_str(BUILTIN_DEF).context("Parse builtin commands toml")?;
    parse_defs(defs)
}

/// Parse and validate command definitions
pub fn parse_defs(mut defs: CommandsDef) -> Result<Vec<Command>> {
    let mut commands = vec![];

    // Verify and build commands
    while let Some(mut def) = defs.commands.pop_front() {
        // Get group names from the template
        let group_names = parse_template_groups(&def.template)
            .context(format!("In template: {}", def.template))?;

        if group_names.len() == 0 {
            return Err(anyhow!("Empty template"));
        }

        let user_input_groups: Vec<&GroupName> = group_names
            .iter()
            .filter(|g| matches!(g.group_type, GroupNameType::UserInput { .. }))
            .collect();

        // Verify all groups are defined
        for group_name in &user_input_groups {
            // Missing group definition
            if !def.groups.contains_key(&group_name.name) {
                return Err(anyhow!(
                    "Command '{}' is missing '{}' group definition.",
                    def.template,
                    group_name.name
                ));
            }
        }

        // Group counts do not match
        if def.groups.keys().count() != user_input_groups.len() {
            let template_group_names: Vec<&str> =
                user_input_groups.iter().map(|g| g.name.as_ref()).collect();
            let groups: Vec<&String> = def.groups.keys().collect();
            return Err(anyhow!(
                "Group counts do not match template={:?} and groups={:?}",
                template_group_names,
                groups,
            ));
        }

        let mut cmd_groups = vec![];

        // Verify each group is correctly defined and build cmd groups
        for group_name in &user_input_groups {
            let name = &group_name.name;
            let group = def.groups.remove(name).expect("Group defined");
            let optional =
                matches!(group_name.group_type, GroupNameType::UserInput { optional } if optional);

            match (group.expect, group.flags) {
                (Some(_expect), Some(_flags)) => {
                    return Err(anyhow!(
                        "Group '{}' defines both expect and flags in '{}'",
                        name,
                        def.template
                    ));
                }
                (None, None) => {
                    return Err(anyhow!(
                        "Group '{}' should define expect or flags in '{}'",
                        name,
                        def.template
                    ));
                }
                (Some(expect), None) => {
                    cmd_groups.push(CmdGroup {
                        name: name.clone(),
                        expect: GroupValue::Single(ValueType::parse(&expect)?),
                        optional,
                    });
                }
                (None, Some(flags)) => {
                    cmd_groups.push(CmdGroup {
                        name: name.clone(),
                        expect: GroupValue::Flags(prepare_flags(flags)?),
                        optional,
                    });
                }
            }
        }

        // TODO: sort cmd_groups to first show required groups

        let build = move |user_input: &HashMap<String, String>| -> String {
            let mut parts = vec![];

            for g in &group_names {
                match g.group_type {
                    // No user input expected
                    GroupNameType::Fixed => {
                        parts.push(g.name.clone());
                    }
                    GroupNameType::UserInput { optional } => match user_input.get(&g.name) {
                        // Replace group with user input
                        Some(value) if !value.is_empty() => {
                            parts.push(value.clone());
                        }
                        // Requires user input -> keep showing the group
                        None if !optional => {
                            parts.push(format!("_{}_", g.name));
                        }
                        // Doesn't require user input or empty value -> ignore
                        _ => {}
                    },
                }
            }

            parts.join("")
        };

        commands.push(Command {
            template: def.template,
            description: def.description,
            groups: cmd_groups,
            build: Box::new(build),
        });
    }

    Ok(commands)
}

fn prepare_flags(mut defs: VecDeque<FlagDef>) -> Result<Vec<Flag>> {
    let mut flags = vec![];

    while let Some(flag_def) = defs.pop_front() {
        let group_names = parse_template_groups(&flag_def.template)
            .context(format!("In flag {}", flag_def.template))?;
        let user_input_groups: Vec<_> = group_names
            .iter()
            .filter(|g| matches!(g.group_type, GroupNameType::UserInput { .. }))
            .collect();

        let expect = match flag_def.expect {
            Some(_) if user_input_groups.len() != 1 => {
                return Err(anyhow!(
                    "Expected one input group for {}",
                    flag_def.template
                ));
            }
            Some(expect) => Some(FlagExpectation {
                value_type: ValueType::parse(&expect)?,
                build: Box::new(move |user_input| {
                    group_names
                        .iter()
                        .map(|g| match g.group_type {
                            GroupNameType::Fixed => &g.name,
                            GroupNameType::UserInput { .. } => user_input,
                        })
                        .collect::<String>()
                }),
            }),
            None => None,
        };

        flags.push(Flag {
            template: flag_def.template,
            description: flag_def.description,
            expect,
            multiple: flag_def.multiple,
            suggest: flag_def.suggest,
        });
    }

    Ok(flags)
}

#[derive(Debug, Clone, PartialEq)]
struct GroupName {
    name: String,
    group_type: GroupNameType,
}

#[derive(Debug, Clone, PartialEq)]
enum GroupNameType {
    UserInput {
        /// For optional group user can skip setting values (e.g flags)
        optional: bool,
    },
    /// Fixed means that this is a static value (e.g command name 'grep')
    Fixed,
}

/// Read command template and return a list of group names.
fn parse_template_groups(template: &str) -> Result<Vec<GroupName>> {
    let mut groups = vec![];
    let mut state = GroupNameType::Fixed;
    let mut optional_started = false;
    let mut current_group = String::new();
    let mut prev_char = ' ';

    for c in template.chars() {
        match c {
            '*' => {
                // Strip bold
            }
            '[' => optional_started = true,
            ']' => {
                if !optional_started {
                    return Err(anyhow!("Unexpected ']' in group '{}'", current_group));
                }
                optional_started = false;
            }
            '_' if prev_char != '\\' => match state {
                GroupNameType::UserInput { .. } => {
                    // Close the group
                    groups.push(GroupName {
                        name: current_group.replace("\\_", "_"),
                        group_type: state,
                    });
                    current_group = String::new();
                    state = GroupNameType::Fixed;
                }
                GroupNameType::Fixed => {
                    // Start user input group
                    if !current_group.is_empty() {
                        // If there is some input already then store it in a separate group
                        groups.push(GroupName {
                            name: current_group.replace("\\_", "_"),
                            group_type: state.clone(),
                        });
                        current_group = String::new();
                    }

                    state = GroupNameType::UserInput {
                        optional: optional_started,
                    };
                }
            },
            c => current_group.push(c),
        }
        prev_char = c;
    }

    if !matches!(state, GroupNameType::Fixed) {
        return Err(anyhow!("Group '{}' is not closed", current_group));
    }

    if !current_group.is_empty() {
        groups.push(GroupName {
            name: current_group.replace("\\_", "_"),
            group_type: state,
        });
    }

    Ok(groups)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_template_groups_grep() {
        let template = "grep [_OPTIONS_] _PATTERN_ _PATH_";
        let names = parse_template_groups(template);
        assert!(names.is_ok(), "No errors");
        assert_eq!(
            vec![
                GroupName {
                    name: "grep ".into(),
                    group_type: GroupNameType::Fixed,
                },
                GroupName {
                    name: "OPTIONS".into(),
                    group_type: GroupNameType::UserInput { optional: true },
                },
                GroupName {
                    name: " ".into(),
                    group_type: GroupNameType::Fixed,
                },
                GroupName {
                    name: "PATTERN".into(),
                    group_type: GroupNameType::UserInput { optional: false },
                },
                GroupName {
                    name: " ".into(),
                    group_type: GroupNameType::Fixed,
                },
                GroupName {
                    name: "PATH".into(),
                    group_type: GroupNameType::UserInput { optional: false },
                },
            ],
            names.ok().unwrap()
        );
    }

    #[test]
    fn parse_template_groups_git_email() {
        let template = "git config [_OPTIONS_] user.email _EMAIL_";
        let names = parse_template_groups(template);
        assert!(names.is_ok(), "Parse failed: {:?}", names.err());
        assert_eq!(
            vec![
                GroupName {
                    name: "git config ".into(),
                    group_type: GroupNameType::Fixed,
                },
                GroupName {
                    name: "OPTIONS".into(),
                    group_type: GroupNameType::UserInput { optional: true },
                },
                GroupName {
                    name: " user.email ".into(),
                    group_type: GroupNameType::Fixed,
                },
                GroupName {
                    name: "EMAIL".into(),
                    group_type: GroupNameType::UserInput { optional: false },
                },
            ],
            names.ok().unwrap()
        );
    }

    #[test]
    fn parse_template_groups_flags() {
        let template = "*-A*_NUM_";
        let names = parse_template_groups(template);
        assert!(names.is_ok(), "Parse failed: {:?}", names.err());
        assert_eq!(
            vec![
                GroupName {
                    name: "-A".into(),
                    group_type: GroupNameType::Fixed,
                },
                GroupName {
                    name: "NUM".into(),
                    group_type: GroupNameType::UserInput { optional: false },
                },
            ],
            names.ok().unwrap()
        );
    }

    #[test]
    fn parse_template_groups_curl() {
        let template = "curl -XPOST -d 'client\\_id=key&client\\_secret=secret' http://localhost?grant=client\\_credentials";
        let names = parse_template_groups(template);
        assert!(names.is_ok(), "Parse failed: {:?}", names.err());
        assert_eq!(
            vec![
                GroupName {
                    name: "curl -XPOST -d 'client_id=key&client_secret=secret' http://localhost?grant=client_credentials".into(),
                    group_type: GroupNameType::Fixed,
                },
            ],
            names.ok().unwrap()
        );
    }

    #[test]
    fn parse_template_groups_err() {
        let template = "grep _PATH";
        let names = parse_template_groups(template);
        assert!(names.is_err(), "Groups should have an error");
        let err_str = format!("{}", names.err().unwrap());
        assert_eq!("Group 'PATH' is not closed", err_str);
    }

    #[test]
    fn parse_defs_ok() {
        let mut groups = HashMap::new();
        groups.insert(
            "PATH".to_string(),
            GroupDef {
                expect: Some("path".into()),
                flags: None,
            },
        );
        groups.insert(
            "OPTIONS".to_string(),
            GroupDef {
                expect: None,
                flags: Some(VecDeque::from(vec![
                    FlagDef {
                        template: "-i".into(),
                        description: "Case insensitive matching".into(),
                        expect: None,
                        multiple: false,
                        suggest: None,
                    },
                    FlagDef {
                        template: "*-A*_NUM_".into(),
                        description: "Print _NUM_ lines after the matched line".into(),
                        expect: Some("number".into()),
                        multiple: false,
                        suggest: None,
                    },
                ])),
            },
        );

        let defs = CommandsDef {
            commands: vec![CommandDef {
                template: "grep [_OPTIONS_] _PATH_".into(),
                description: "Find lines in a file (*grep*)".into(),
                groups,
            }]
            .into(),
        };

        let commands = parse_defs(defs);
        assert!(
            commands.is_ok(),
            "Parse defs is ok (err={:?})",
            commands.err()
        );

        let commands = commands.ok().unwrap();
        assert_eq!(1, commands.len());
        let cmd = &commands[0];
        assert_eq!("grep [_OPTIONS_] _PATH_", cmd.template);
        assert_eq!(2, cmd.groups.len(), "Groups len");

        // TODO: assert flags

        let mut user_input = HashMap::new();
        user_input.insert("PATH".to_string(), "./one".to_string());

        let result = (cmd.build)(&user_input);
        assert_eq!("grep  ./one", result);
    }

    #[test]
    fn parse_defs_inline_group() {
        let mut groups = HashMap::new();
        groups.insert(
            "VALUE".to_string(),
            GroupDef {
                expect: Some("string".into()),
                flags: None,
            },
        );

        let defs = CommandsDef {
            commands: vec![CommandDef {
                template: "curl http://localhost?one=_VALUE_".into(),
                description: "Get something".into(),
                groups,
            }]
            .into(),
        };

        let commands = parse_defs(defs);
        assert!(
            commands.is_ok(),
            "Parse defs is ok (err={:?})",
            commands.err()
        );

        let commands = commands.ok().unwrap();
        assert_eq!(1, commands.len());
        let cmd = &commands[0];
        assert_eq!(1, cmd.groups.len(), "Groups len");

        let mut user_input = HashMap::new();
        user_input.insert("VALUE".to_string(), "value".to_string());

        let result = (cmd.build)(&user_input);
        assert_eq!("curl http://localhost?one=value", result);
    }

    #[test]
    fn parse_defs_missing_group() {
        let mut groups = HashMap::new();
        groups.insert(
            "PATH".to_string(),
            GroupDef {
                expect: Some("path".into()),
                flags: None,
            },
        );

        let defs = CommandsDef {
            commands: vec![CommandDef {
                template: "grep [_OPTIONS_] _PATH_".into(),
                description: "Find lines in a file (*grep*)".into(),
                groups,
            }]
            .into(),
        };

        let commands = parse_defs(defs);
        assert!(commands.is_err(), "Parse defs is err");
        let err_str = format!("{}", commands.err().unwrap());
        assert_eq!(
            "Command 'grep [_OPTIONS_] _PATH_' is missing 'OPTIONS' group definition.",
            err_str
        );
    }

    #[test]
    fn parse_defs_missing_expect_and_flags() {
        let mut groups = HashMap::new();
        groups.insert(
            "OPTIONS".to_string(),
            GroupDef {
                expect: None,
                flags: None,
            },
        );

        let defs = CommandsDef {
            commands: vec![CommandDef {
                template: "grep [_OPTIONS_]".into(),
                description: "Find lines in a file (*grep*)".into(),
                groups,
            }]
            .into(),
        };

        let commands = parse_defs(defs);
        assert!(commands.is_err(), "Parse defs is err");
        let err_str = format!("{}", commands.err().unwrap());
        assert_eq!(
            "Group 'OPTIONS' should define expect or flags in 'grep [_OPTIONS_]'",
            err_str
        );
    }
}
