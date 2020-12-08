use anyhow::{anyhow, Result};
use std::fmt::Display;
use std::io::{stdin, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{clear, cursor};

pub trait Choice {
    /// User visible text
    type Text: Display;

    /// Get a reference to the text
    fn text(&self) -> &Self::Text;
}

const ROWS_PLACEHOLDER: u16 = 5;

pub struct Readline {
    // TODO: which type of input to expect String, Number, Path, ..
    expect_input: (),
    prefix: String,
}

impl Readline {
    pub fn new() -> Self {
        Self {
            expect_input: (),
            prefix: "$".into(),
        }
    }

    pub fn prefix(mut self, value: impl Into<String>) -> Self {
        self.prefix = value.into();
        self
    }

    /// Read a character mutate the input and return
    fn simple(&mut self, input: &mut String) -> Result<Key> {
        let stdout = stdout();
        let mut stdout = stdout.lock().into_raw_mode()?;
        let stdin = stdin();

        write!(stdout, "{} {}", self.prefix, input)?;
        stdout.flush()?;

        for key in stdin.keys() {
            let key = key?;

            match key {
                Key::Ctrl('c') => {
                    write!(stdout, "{}", clear::CurrentLine)?;
                    write!(stdout, "\r{} {}\n\r", self.prefix, input)?;
                    stdout.flush()?;
                    return Err(anyhow!("Terminated"));
                }
                Key::Char('\n') => {}
                Key::Char(c) => {
                    // TODO: deny unexpected_input
                    input.push(c);
                }
                Key::Backspace => {
                    input.pop();
                }
                _ => {}
            }

            write!(stdout, "{}", clear::CurrentLine)?;
            write!(stdout, "\r{} {}", self.prefix, input)?;
            stdout.flush()?;

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
        let stdout = stdout();
        let mut stdout = stdout.lock().into_raw_mode()?;
        let mut i = 0;

        loop {
            if i > 0 {
                write!(stdout, "{}", cursor::Up(ROWS_PLACEHOLDER))?;
            }
            i += 1;

            let choices = get_choices(&input);
            if choices.len() <= selected {
                selected = (choices.len() - 1).max(0);
            }
            render_choices(&mut stdout, &choices, selected)?;

            match self.simple(&mut input)? {
                Key::Char('\n') => {
                    break;
                }
                Key::Up if selected > 0 => {
                    selected -= 1;
                }
                Key::Down if selected < (choices.len() - 1) => {
                    selected += 1;
                }
                Key::Ctrl('d') => {
                    // Choose None
                    write!(stdout, "\n\r")?;
                    stdout.flush()?;

                    return Ok(None);
                }
                _ => {}
            }
        }

        write!(stdout, "\n\r")?;
        stdout.flush()?;

        let choices = get_choices(&input);
        Ok(choices.get(selected).cloned())
    }

    /// Read line
    pub fn line(&mut self) -> Result<String> {
        let mut input = String::new();
        loop {
            match self.simple(&mut input)? {
                Key::Char('\n') => {
                    break;
                }
                Key::Ctrl('d') => {
                    break;
                }
                _ => {}
            }
        }

        let stdout = stdout();
        let mut stdout = stdout.lock().into_raw_mode()?;
        write!(stdout, "\n\r")?;
        stdout.flush()?;
        Ok(input)
    }
}

fn render_choices<'a, C>(stdout: &mut dyn Write, choices: &Vec<&C>, selected: usize) -> Result<()>
where
    C: Choice,
{
    let total = choices.len();
    let empty_rows = ROWS_PLACEHOLDER as usize - total;

    for i in 0..empty_rows {
        write!(stdout, "{}\n\r", clear::CurrentLine)?;
    }

    for (i, choice) in choices.iter().enumerate() {
        write!(stdout, "{}", clear::CurrentLine);
        if i == selected {
            write!(stdout, "> {}", choice.text())?;
        } else {
            write!(stdout, "  {}", choice.text())?;
        }
        write!(stdout, "\n\r")?;
    }

    Ok(())
}
