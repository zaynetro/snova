use anyhow::{anyhow, Result};
use std::fmt::Display;
use std::io::{stdin, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::{clear, cursor};

use crate::cmd::ValueType;

pub trait Choice {
    /// User visible text
    type Text: Display;

    /// Get a reference to the text
    fn text(&self) -> &Self::Text;
}

impl Choice for &'static str {
    type Text = &'static str;

    fn text(&self) -> &Self::Text {
        self
    }
}

const ROWS_PLACEHOLDER: u16 = 5;

pub struct Readline<'s> {
    expect_input: Option<ValueType>,
    prefix: String,
    stdout: &'s mut dyn Write,
}

impl<'s> Readline<'s> {
    pub fn new(stdout: &'s mut dyn Write) -> Self {
        Self {
            expect_input: None,
            prefix: "$".into(),
            stdout,
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

    /// Read a character mutate the input and return
    fn simple(&mut self, input: &mut String) -> Result<Key> {
        let stdin = stdin();

        write!(self.stdout, "{}", clear::CurrentLine)?;
        write!(self.stdout, "\r{} {}", self.prefix, input)?;
        self.stdout.flush()?;

        for key in stdin.keys() {
            let key = key?;

            match key {
                Key::Ctrl('c') => {
                    write!(self.stdout, "{}", clear::CurrentLine)?;
                    write!(self.stdout, "\r{} {}\n\r", self.prefix, input)?;
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
            write!(self.stdout, "\r{} {}", self.prefix, input)?;
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
        let mut i = 0;

        loop {
            if i > 0 {
                write!(self.stdout, "{}\r", cursor::Up(ROWS_PLACEHOLDER))?;
            }
            i += 1;

            let choices = get_choices(&input);
            if choices.len() <= selected {
                selected = choices.len().max(1) - 1;
            }
            render_choices(&mut self.stdout, &choices, selected)?;

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

        write!(self.stdout, "\n\r")?;
        self.stdout.flush()?;

        let choices = get_choices(&input);
        Ok(choices.get(selected).cloned())
    }

    /// Read line
    pub fn line(&mut self) -> Result<String> {
        let mut input = String::new();
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

fn render_choices<'a, C>(stdout: &mut dyn Write, choices: &Vec<&C>, selected: usize) -> Result<()>
where
    C: Choice,
{
    let total = choices.len();
    let empty_rows = (ROWS_PLACEHOLDER as isize - total as isize).max(0);

    for _ in 0..empty_rows {
        write!(stdout, "{}\n\r", clear::CurrentLine)?;
    }

    // TODO: scroll list if selected is not on the screen

    for (i, choice) in choices.iter().enumerate().take(ROWS_PLACEHOLDER as usize) {
        write!(stdout, "{}", clear::CurrentLine)?;
        if i == selected {
            write!(stdout, "> {}", choice.text())?;
        } else {
            write!(stdout, "  {}", choice.text())?;
        }
        write!(stdout, "\n\r")?;
    }

    Ok(())
}
