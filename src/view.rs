use anyhow::{anyhow, Result};
use std::io::{stdin, Write};
use termion::event::Key;
use termion::{clear, style};
use termion::{cursor, input::TermRead};

use crate::cmd::ValueType;

/// Size of autocomplete window
const AUTOCOMPLETE_ROWS: u16 = 8;

pub trait Choice {
    /// Get a reference to the text
    fn text(&self) -> &str;
}

impl Choice for () {
    fn text(&self) -> &str {
        ""
    }
}

pub struct Readline<'s> {
    expect_input: Option<ValueType>,
    prefix: String,
    stdout: &'s mut dyn Write,
    help: Option<String>,
    scroll_offset: usize,
}

impl<'s> Readline<'s> {
    pub fn new(stdout: &'s mut dyn Write) -> Self {
        Self {
            expect_input: None,
            prefix: "$".into(),
            stdout,
            help: None,
            scroll_offset: 0,
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

    /// Return a choice from one of the autocomplete options.
    /// Returns None if input was interrupted (e.g with ctrl-d).
    pub fn choice<'c, C>(
        &mut self,
        autocomplete: impl Fn(&str) -> Vec<&'c C>,
    ) -> Result<Option<&'c C>>
    where
        C: Choice,
    {
        let (choice, _) = self.run(Some(autocomplete))?;
        Ok(choice)
    }

    /// Read a single line
    pub fn line<'c>(&mut self) -> Result<String> {
        let (_, text) = self.run(None::<Box<dyn Fn(&str) -> Vec<&'c ()>>>)?;
        Ok(text)
    }

    /// Mutates the input based on the keys from stdin. It then returns the key
    /// for post processing.
    fn read_key(&mut self, input: &mut String) -> Result<Key> {
        let stdin = stdin();
        for key in stdin.keys() {
            let key = key?;
            match key {
                Key::Ctrl('c') => {
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
                // TODO:
                Key::Left => {}
                // TODO:
                Key::Right => {}
                // TODO:
                Key::Delete => {}
                _ => {}
            }

            return Ok(key);
        }

        Ok(Key::Char('\n'))
    }

    fn run<'c, A, C>(&mut self, autocomplete: Option<A>) -> Result<(Option<&'c C>, String)>
    where
        A: Fn(&str) -> Vec<&'c C>,
        C: Choice,
    {
        let mut input = String::new();
        let mut selected: usize = 0;
        let reserve_rows = {
            // User input row
            let mut rows = 1;
            // Help row above user input
            if self.help.is_some() {
                rows += 1;
            }
            // Autocomplete rows
            if autocomplete.is_some() {
                rows += AUTOCOMPLETE_ROWS;
            }
            rows
        };
        let mut choices = vec![];

        // TODO: in case of error clean up always
        let choice = loop {
            write!(self.stdout, "{}\r", clear::AfterCursor)?;

            // Render autocomplete choices
            if let Some(autocomplete) = &autocomplete {
                choices = autocomplete(&input);

                if selected >= choices.len() {
                    selected = choices.len().max(1) - 1;
                }
                self.render_choices(&choices, selected)?;
            }

            // Display help
            if let Some(ref help) = self.help {
                write!(self.stdout, "{}\r\n", fmt_text(help))?;
            }

            // Display user input
            write!(self.stdout, "{} {}", fmt_text(&self.prefix), input)?;
            self.stdout.flush()?;

            let key = match self.read_key(&mut input) {
                Ok(key) => key,
                Err(e) => break Err(e),
            };

            match key {
                Key::Char('\n') => {
                    if autocomplete.is_some() {
                        // Require a choice
                        if let Some(c) = choices.get(selected) {
                            break Ok(Some(c.clone()));
                        }
                    } else if self.expect_input.is_some() && !input.is_empty() {
                        // When expecting an input require it to be non-empty
                        break Ok(None);
                    } else if self.expect_input.is_none() {
                        // When non expecting an input simply return
                        break Ok(None);
                    }
                }
                Key::Up | Key::Ctrl('j') if selected > 0 => {
                    selected -= 1;

                    // If cursor moved outsize of visible window then scroll
                    if selected < self.scroll_offset {
                        self.scroll_offset -= 1;
                    }
                }
                Key::Down | Key::Ctrl('k') if selected < (choices.len() - 1) => {
                    selected += 1;

                    // If cursor moved outsize of visible window then scroll
                    if selected >= AUTOCOMPLETE_ROWS as usize + self.scroll_offset {
                        self.scroll_offset += 1;
                    }
                }
                Key::Ctrl('d') => {
                    break Ok(None);
                }
                _ => {}
            }

            if reserve_rows > 1 {
                write!(self.stdout, "{}\r", cursor::Up(reserve_rows - 1))?;
            }
        };

        if reserve_rows > 1 {
            write!(self.stdout, "{}\r", cursor::Up(reserve_rows - 1))?;
        }
        write!(self.stdout, "{}\r", clear::AfterCursor)?;
        self.stdout.flush()?;

        let choice = choice?;
        Ok((choice, input))
    }

    fn render_choices<C>(&mut self, choices: &Vec<&C>, selected: usize) -> Result<()>
    where
        C: Choice,
    {
        let total = choices.len();
        let size = AUTOCOMPLETE_ROWS - 1;
        let empty_rows = (size as isize - total as isize).max(0);

        for _ in 0..empty_rows {
            write!(self.stdout, "{}\n\r", clear::CurrentLine)?;
        }

        for (i, choice) in choices
            .iter()
            .enumerate()
            .skip(self.scroll_offset)
            .take(size as usize)
        {
            write!(self.stdout, "{}", clear::CurrentLine)?;
            if i == selected {
                write!(
                    self.stdout,
                    "> {}{}{}",
                    style::Bold,
                    fmt_text(choice.text()),
                    style::Reset
                )?;
            } else {
                write!(self.stdout, "  {}", fmt_text(choice.text()))?;
            }
            write!(self.stdout, "\n\r")?;
        }

        write!(
            self.stdout,
            "  {}{}/{}{}\n\r",
            style::Italic,
            selected + 1,
            total,
            style::NoItalic
        )?;
        Ok(())
    }
}

#[derive(Default)]
struct FmtState {
    /// Bold text has started
    bold: bool,
    /// Underline text has started
    underline: bool,
}

pub fn fmt_text(text: impl AsRef<str>) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmt_text_ok() {
        assert_eq!(
            format!(
                "Hello {}UNDERLINE{} and {}bold{}",
                style::Underline,
                style::NoUnderline,
                style::Bold,
                style::Reset
            ),
            fmt_text("Hello _UNDERLINE_ and *bold*")
        );

        assert_eq!(
            format!("inline={}underline{}", style::Underline, style::NoUnderline),
            fmt_text("inline=_underline_")
        );
    }
}
