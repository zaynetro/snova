use anyhow::{anyhow, Result, Context};
use std::fmt::Display;
use std::io::{stdin, Write};
use termion::input::TermRead;
use termion::{clear, style};
use termion::{event::Key, terminal_size};

use crate::cmd::ValueType;

pub trait Choice {
    /// Get a reference to the text
    fn text(&self) -> &str;
}

impl Choice for &'static str {
    fn text(&self) -> &str {
        self
    }
}

pub struct Readline<'s> {
    expect_input: Option<ValueType>,
    prefix: String,
    stdout: &'s mut dyn Write,
    help: Option<String>,
}

impl<'s> Readline<'s> {
    pub fn new(stdout: &'s mut dyn Write) -> Self {
        Self {
            expect_input: None,
            prefix: "$".into(),
            stdout,
            help: None,
        }
    }

    pub fn prefix(mut self, value: impl Into<String>) -> Self {
        self.prefix = value.into();
        self
    }

    pub fn expect(mut self, expect_type: ValueType) -> Self {
        self.expect_input = Some(expect_type);
        self
    }

    pub fn help(mut self, value: impl Into<String>) -> Self {
        self.help = Some(value.into());
        self
    }

    /// Read a character mutate the input and return
    fn simple(&mut self, input: &mut String) -> Result<Key> {
        let stdin = stdin();

        write!(self.stdout, "{}", clear::CurrentLine)?;
        write!(self.stdout, "\r{} {}", fmt_text(&self.prefix), input)?;
        self.stdout.flush()?;

        for key in stdin.keys() {
            let key = key?;

            match key {
                Key::Ctrl('c') => {
                    write!(self.stdout, "{}", clear::CurrentLine)?;
                    write!(self.stdout, "\r{} {}\n\r", fmt_text(&self.prefix), input)?;
                    self.stdout.flush()?;
                    return Err(anyhow!("Terminated"));
                }
                Key::Char('\n') => {}
                Key::Char(c) => match &self.expect_input {
                    Some(expect) if expect.is_valid_char(c) => {
                        input.push(c);
                    }
                    Some(_) => {}
                    None => {
                        input.push(c);
                    }
                },
                Key::Backspace => {
                    input.pop();
                }
                _ => {}
            }

            write!(self.stdout, "{}", clear::CurrentLine)?;
            write!(self.stdout, "\r{} {}", fmt_text(&self.prefix), input)?;
            self.stdout.flush()?;

            return Ok(key);
        }

        Ok(Key::Char('\n'))
    }

    /// Autocomplete
    pub fn autocomplete<'a, C>(
        mut self,
        get_choices: impl Fn(&String) -> Vec<&'a C>,
    ) -> Result<Option<&'a C>>
    where
        C: Choice,
    {
        let mut input = String::new();
        let mut selected: usize = 0;

        loop {
            write!(self.stdout, "{}\r", clear::All)?;

            let choices = get_choices(&input);
            if choices.len() <= selected {
                selected = choices.len().max(1) - 1;
            }
            render_choices(&mut self.stdout, &choices, selected)?;

            if let Some(ref help) = self.help {
                write!(self.stdout, "\n{}\r\n", fmt_text(help))?;
            }

            match self.simple(&mut input)? {
                Key::Char('\n') => {
                    break;
                }
                Key::Up if selected > 0 => {
                    selected -= 1;
                }
                Key::Down | Key::Ctrl('k') if selected < (choices.len() - 1) => {
                    selected += 1;
                }
                Key::Ctrl('d') => {
                    // Choose None
                    write!(self.stdout, "\n\r")?;
                    self.stdout.flush()?;

                    return Ok(None);
                }
                _ => {}
            }
        }

        write!(self.stdout, "{}\r", clear::All)?;
        self.stdout.flush()?;

        let choices = get_choices(&input);
        Ok(choices.get(selected).cloned())
    }

    /// Read line
    pub fn line(&mut self) -> Result<String> {
        let mut input = String::new();

        if let Some(ref help) = self.help {
            write!(self.stdout, "\n{}\r\n", fmt_text(help))?;
        }

        loop {
            match self.simple(&mut input)? {
                Key::Char('\n') if !input.is_empty() => {
                    break;
                }
                Key::Ctrl('d') => {
                    break;
                }
                _ => {}
            }
        }

        write!(self.stdout, "\n\r")?;
        self.stdout.flush()?;
        Ok(input)
    }
}

fn render_choices<'a, C>(
    stdout: &mut dyn Write,
    choices: &Vec<&C>,
    selected: usize,
) -> Result<()>
where
    C: Choice,
{
    let total = choices.len();
    let size = terminal_size().context("Terminal size")?;
    let empty_rows = (size.1 as isize - total as isize).max(0);

    for _ in 0..empty_rows {
        write!(stdout, "{}\n\r", clear::CurrentLine)?;
    }

    // TODO: scroll list if selected is not on the screen

    for (i, choice) in choices.iter().enumerate().take(size.1 as usize) {
        write!(stdout, "{}", clear::CurrentLine)?;
        if i == selected {
            write!(stdout, "> {}", fmt_text(choice.text()))?;
        } else {
            write!(stdout, "  {}", fmt_text(choice.text()))?;
        }
        write!(stdout, "\n\r")?;
    }

    Ok(())
}

#[derive(Default)]
struct FmtState {
    /// Bold text has started
    bold: bool,
    /// Underline text has started
    underline: bool,
}

pub fn fmt_text(text: impl AsRef<str>) -> impl Display {
    let text = text.as_ref();
    let mut result = String::new();
    let mut state = FmtState::default();

    for c in text.chars() {
        match c {
            '*' => {
                if state.bold {
                    // End bold
                    // Somehow NoBold doesn't work properly hence using Reset for now
                    // result.push_str(style::NoBold.as_ref());
                    result.push_str(style::Reset.as_ref());
                } else {
                    // Start bold
                    result.push_str(style::Bold.as_ref());
                }

                state.bold = !state.bold;
            }
            '_' => {
                if state.underline {
                    // End underline
                    result.push_str(style::NoUnderline.as_ref());
                } else {
                    // Start underline
                    result.push_str(style::Underline.as_ref());
                }

                state.underline = !state.underline;
            }
            _ => {
                result.push(c);
            }
        }
    }

    // Clean styles if not closed properly
    if state.bold {
        // Somehow NoBold doesn't work properly hence using Reset for now
        // result.push_str(style::NoBold.as_ref());
        result.push_str(style::Reset.as_ref());
    }

    if state.underline {
        result.push_str(style::NoUnderline.as_ref());
    }

    result
}
