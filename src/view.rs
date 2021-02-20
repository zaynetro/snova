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

impl Choice for String {
    fn text(&self) -> &str {
        &self
    }
}

impl<'a, C> Choice for &'a C
where
    C: Choice,
{
    fn text(&self) -> &str {
        (*self).text()
    }
}

pub struct Readline<'s> {
    expect_input: Option<ValueType>,
    prefix: String,
    stdout: &'s mut dyn Write,
    help: Option<String>,
    scroll_offset: usize,
    /// Cursor horizontal position
    cursor: usize,
}

enum AutocompleteMode<A> {
    /// Enable autocompletion
    Enabled {
        autocomplete: A,
        allow_user_input: bool,
    },
    /// No completion
    None,
}

impl<A> AutocompleteMode<A> {
    fn enabled(&self) -> bool {
        matches!(self, AutocompleteMode::Enabled { .. })
    }
}

impl<'s> Readline<'s> {
    pub fn new(stdout: &'s mut dyn Write) -> Self {
        Self {
            expect_input: None,
            prefix: "$".into(),
            stdout,
            help: None,
            scroll_offset: 0,
            cursor: 0,
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
        autocomplete: impl AutoComplete<'c, C = C>,
    ) -> Result<Option<&'c C>>
    where
        C: Choice,
    {
        let (choice, _) = self.run(AutocompleteMode::Enabled {
            autocomplete,
            allow_user_input: false,
        })?;
        Ok(choice)
    }

    /// Return a choice from one of the autocomplete options and a user input.
    /// This can be used when user is not required to pick an option
    /// but instead could provide a custom value.
    pub fn suggest<'c, C>(
        &mut self,
        autocomplete: impl AutoComplete<'c, C = C>,
    ) -> Result<(Option<&'c C>, String)>
    where
        C: Choice,
    {
        self.run(AutocompleteMode::Enabled {
            autocomplete,
            allow_user_input: true,
        })
    }

    /// Read a single line
    pub fn line<'c>(&mut self) -> Result<String> {
        let (_, text) = self.run(AutocompleteMode::None::<FixedComplete<'c, String>>)?;
        Ok(text)
    }

    /// Mutates the input based on the keys from stdin. It then returns the key
    /// for post processing.
    fn read_key(
        &mut self,
        keys: impl Iterator<Item = std::result::Result<Key, std::io::Error>>,
        input: &mut String,
    ) -> Result<Key> {
        for key in keys {
            let key = key?;
            match key {
                Key::Ctrl('c') => {
                    return Err(anyhow!("Terminated"));
                }
                Key::Ctrl('u') => {
                    // Remove chars before the cursor
                    input.drain(0..self.cursor);
                    self.cursor = 0;
                }
                Key::Char('\n') => {}
                Key::Char(c) => match &self.expect_input {
                    Some(expect) if !expect.is_valid_char(c) => {}
                    _ => {
                        input.insert(self.cursor, c);
                        self.cursor += 1;
                    }
                },
                Key::Backspace => {
                    self.cursor = self.cursor.saturating_sub(1);
                    if self.cursor == 0 {
                        input.pop();
                    } else {
                        input.drain(self.cursor..self.cursor + 1);
                    }
                }
                Key::Left => {
                    self.cursor = self.cursor.saturating_sub(1);
                }
                Key::Right if self.cursor < input.len() => {
                    self.cursor += 1;
                }
                _ => {}
            }

            return Ok(key);
        }

        Ok(Key::Char('\n'))
    }

    fn run<'c, A, C>(
        &mut self,
        mut autocomplete: AutocompleteMode<A>,
    ) -> Result<(Option<&'c C>, String)>
    where
        C: Choice,
        A: AutoComplete<'c, C = C>,
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
            if autocomplete.enabled() {
                rows += AUTOCOMPLETE_ROWS;
            }
            rows
        };
        let mut choices = vec![];
        let mut choices_len = 0;
        let mut keys = stdin().keys();

        // TODO: in case of error clean up always
        let choice = loop {
            write!(self.stdout, "{}\r", clear::AfterCursor)?;

            // Render autocomplete choices
            if let AutocompleteMode::Enabled {
                autocomplete,
                allow_user_input,
            } = &mut autocomplete
            {
                choices = autocomplete.list(&input);
                choices_len = choices.len();

                if *allow_user_input && !input.is_empty() {
                    choices_len += 1;
                }

                if selected >= choices_len {
                    selected = choices_len.saturating_sub(1);
                }

                let mut view_choices: Vec<&str> = choices.iter().map(|c| c.text()).collect();
                if *allow_user_input && !input.is_empty() {
                    view_choices.push(&input);
                }

                self.render_choices(&view_choices, selected)?;
            }

            // Display help
            if let Some(ref help) = self.help {
                write!(self.stdout, "{}\r\n", fmt_text(help))?;
            }

            // Display user input
            write!(self.stdout, "{} {} ", fmt_text(&self.prefix), input,)?;
            // Cursor position is 1 based.
            let cursor_left = input.len().saturating_sub(self.cursor) + 1;
            write!(self.stdout, "{}", cursor::Left(cursor_left as u16))?;
            self.stdout.flush()?;

            let key = match self.read_key(&mut keys, &mut input) {
                Ok(key) => key,
                Err(e) => break Err(e),
            };

            match key {
                Key::Char('\n') => {
                    if let AutocompleteMode::Enabled {
                        allow_user_input, ..
                    } = autocomplete
                    {
                        let choice = choices.get(selected).cloned();
                        if allow_user_input {
                            // It is fine not to have a choice when user can
                            // input their own value
                            break Ok(choice);
                        } else if choice.is_some() {
                            // Require a choice when running in a strict mode
                            break Ok(choice);
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
                Key::Down | Key::Ctrl('k') if selected < (choices_len.saturating_sub(1)) => {
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

    fn render_choices(&mut self, choices: &[&str], selected: usize) -> Result<()> {
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
                    fmt_text(choice),
                    style::Reset
                )?;
            } else {
                write!(self.stdout, "  {}", fmt_text(choice))?;
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

pub trait AutoComplete<'c> {
    type C: Choice;

    fn list(&mut self, input: &str) -> Vec<&'c Self::C>;
}

/// Autocomplete from a fixed set of options
pub struct FixedComplete<'c, C> {
    options: &'c Vec<C>,
}

impl<'c, C> FixedComplete<'c, C>
where
    C: Choice,
{
    pub fn new(options: &'c Vec<C>) -> Self {
        Self { options }
    }
}

impl<'c, C> AutoComplete<'c> for FixedComplete<'c, C>
where
    C: Choice,
{
    type C = C;

    fn list(&mut self, input: &str) -> Vec<&'c C> {
        self.options
            .iter()
            .filter(|o| o.text().to_lowercase().contains(&input.to_lowercase()))
            .collect()
    }
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
