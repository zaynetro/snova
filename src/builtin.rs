use crate::cmd::*;

pub fn all() -> Vec<Command> {
    vec![
        Command {
            template: "grep [_OPTIONS_] _PATTERN_ _PATH_".into(),
            description: "Find lines in a file (*grep*)".into(),
            groups: vec![
                CmdGroup {
                    name: "PATTERN".into(),
                    expect: GroupValue::Single(ValueType::String),
                },
                CmdGroup {
                    name: "PATH".into(),
                    expect: GroupValue::Single(ValueType::Path),
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
                            template: "*-A* _NUM_".into(),
                            description: "Print _NUM_ lines after the matched line".into(),
                            expect: Some(FlagExpectation {
                                build: Box::new(|num| format!("-A{}", num)),
                                value_type: ValueType::Number,
                            }),
                        },
                        Flag {
                            template: "*-B* _NUM_".into(),
                            description: "Print _NUM_ lines before the matched line".into(),
                            expect: Some(FlagExpectation {
                                build: Box::new(|num| format!("-B{}", num)),
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
                        .unwrap_or(&"_OPTIONS_".to_string())
                        .trim(),
                    options
                        .get("PATTERN")
                        .unwrap_or(&"_PATTERN_".to_string())
                        .trim(),
                    options.get("PATH").unwrap_or(&"_PATH_".to_string()).trim(),
                )
            }),
        },
        Command {
            template: "find _PATH_ _EXPRESSION_".into(),
            description: "Find files or directories (*find*)".into(),
            groups: vec![
                CmdGroup {
                    name: "PATH".into(),
                    expect: GroupValue::Single(ValueType::Path),
                },
                CmdGroup {
                    name: "EXPRESSION".into(),
                    // TODO: how can we require that at least one flag shoud be selected?
                    expect: GroupValue::Flags(vec![Flag {
                        template: "*-iname* _PATTERN_".into(),
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
                    options.get("PATH").unwrap_or(&"_PATH_".to_string()).trim(),
                    options
                        .get("EXPRESSION")
                        .unwrap_or(&"_EXPRESSION_".to_string())
                        .trim(),
                )
            }),
        },
        Command {
            template: "git config [_OPTIONS_] user.email _EMAIL_".into(),
            description: "Set git email address (*git*)".into(),
            groups: vec![
                CmdGroup {
                    name: "EMAIL".into(),
                    expect: GroupValue::Single(ValueType::Path),
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
                    options.get("OPTIONS").unwrap_or(&"_OPTIONS_".to_string()).trim(),
                    options
                        .get("EMAIL")
                        .unwrap_or(&"_EMAIL_".to_string())
                        .trim(),
                )
            }),
        },
        Command {
            template: "curl [_OPTIONS_] _URL_".into(),
            description: "Send an HTTP request (*curl*)".into(),
            groups: vec![
                CmdGroup {
                    name: "URL".into(),
                    expect: GroupValue::Single(ValueType::String),
                },
                CmdGroup {
                    name: "OPTIONS".into(),
                    expect: GroupValue::Flags(vec![Flag {
                        template: "*-H* _VALUE_".into(),
                        description: "Include header (e.g -H \"Content-Type: application/json\")".into(),
                        expect: Some(FlagExpectation {
                            build: Box::new(|value| format!("-H {}", value)),
                            value_type: ValueType::String,
                        })
                    }, Flag {
                        template: "*-X* _METHOD_".into(),
                        description: "Specify a request method to use".into(),
                        expect: Some(FlagExpectation {
                            build: Box::new(|method| format!("-X{}", method)),
                            value_type: ValueType::String,
                        })
                    }, Flag {
                        template: "-v".into(),
                        description: "Verbose logging".into(),
                        expect: None,
                    }, Flag {
                        template: "*-d* _DATA_".into(),
                        description: "Specify request payload (use '@myfile.txt' to read data from file)".into(),
                        expect: Some(FlagExpectation {
                            build: Box::new(|data| format!("-d {}", data)),
                            value_type: ValueType::String,
                        })
                    }, Flag {
                        template: "-L".into(),
                        description: "Follow redirects".into(),
                        expect: None,
                    }]),
                },
            ],
            build: Box::new(|options| {
                format!(
                    "curl {} {}",
                    options.get("OPTIONS").unwrap_or(&"_OPTIONS_".to_string()).trim(),
                    options
                        .get("URL")
                        .unwrap_or(&"_URL_".to_string())
                        .trim(),
                )
            }),
        },
    ]
}
