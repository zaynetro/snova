use crate::cmd::*;

pub fn all() -> Vec<Command> {
    vec![
        Command {
            template: "grep [OPTIONS] PATTERN PATH".into(),
            description: "Find lines in a file".into(),
            groups: vec![
                CmdGroup {
                    name: "PATTERN".into(),
                    expect: GroupValue::String,
                },
                CmdGroup {
                    name: "PATH".into(),
                    expect: GroupValue::Path,
                },
                CmdGroup {
                    name: "OPTIONS".into(),
                    expect: GroupValue::Flags(vec![
                        Flag {
                            template: "-i".into(),
                            description: "Case insensitive matching".into(),
                            expect: None,
                        },
                        Flag {
                            template: "-v".into(),
                            description: "Invert match (return non-matching lines)".into(),
                            expect: None,
                        },
                        Flag {
                            template: "-A _NUM_".into(),
                            description: "Print _NUM_ lines after the matched line".into(),
                            expect: Some(FlagExpectation {
                                build: Box::new(|num| format!("-A {}", num)),
                                value_type: ValueType::Number,
                            }),
                        },
                        Flag {
                            template: "-B _NUM_".into(),
                            description: "Print _NUM_ lines before the matched line".into(),
                            expect: Some(FlagExpectation {
                                build: Box::new(|num| format!("-B {}", num)),
                                value_type: ValueType::Number,
                            }),
                        },
                        Flag {
                            template: "-r".into(),
                            description: "Search files recursively".into(),
                            expect: None,
                        },
                    ]),
                },
            ],
            build: Box::new(|options| {
                format!(
                    "grep {} {} {}",
                    options
                        .get("OPTIONS")
                        .unwrap_or(&"OPTIONS".to_string())
                        .trim(),
                    options
                        .get("PATTERN")
                        .unwrap_or(&"PATTERN".to_string())
                        .trim(),
                    options.get("PATH").unwrap_or(&"PATH".to_string()).trim(),
                )
            }),
        },
        Command {
            template: "find PATH EXPRESSION".into(),
            description: "Find files or directories".into(),
            groups: vec![
                CmdGroup {
                    name: "PATH".into(),
                    expect: GroupValue::Path,
                },
                CmdGroup {
                    name: "EXPRESSION".into(),
                    // TODO: how can we require that at least one flag shoud be selected?
                    expect: GroupValue::Flags(vec![Flag {
                        template: "-iname _PATTERN_".into(),
                        description: "File name pattern".into(),
                        expect: Some(FlagExpectation {
                            build: Box::new(|pattern| format!("-iname {}", pattern)),
                            value_type: ValueType::String,
                        }),
                    }]),
                },
            ],
            build: Box::new(|options| {
                format!(
                    "find {} {}",
                    options.get("PATH").unwrap_or(&"PATH".to_string()).trim(),
                    options
                        .get("EXPRESSION")
                        .unwrap_or(&"EXPRESSION".to_string())
                        .trim(),
                )
            }),
        },
        Command {
            template: "git config [OPTIONS] user.email EMAIL".into(),
            description: "Set git email address".into(),
            groups: vec![
                CmdGroup {
                    name: "EMAIL".into(),
                    expect: GroupValue::String,
                },
                CmdGroup {
                    name: "OPTIONS".into(),
                    expect: GroupValue::Flags(vec![Flag {
                        template: "--global".into(),
                        description: " Write to global *~/.gitconfig* file rather than the repository *.git/config*".into(),
                        expect: None,
                    }]),
                },
            ],
            build: Box::new(|options| {
                format!(
                    "git config {} user.email {}",
                    options.get("OPTIONS").unwrap_or(&"OPTIONS".to_string()).trim(),
                    options
                        .get("EMAIL")
                        .unwrap_or(&"EMAIL".to_string())
                        .trim(),
                )
            }),
        },
    ]
}
