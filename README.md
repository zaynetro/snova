# Snova

A CLI tool to help build you a command you forgot.


```
➤ snova
Command: grep [OPTIONS] PATTERN PATH



> Invert match (return non-matching lines)
  Print NUM lines after the matched line
  Print NUM lines before the matched line
  Search files recursively
  1/4
grep -i PATTERN PATH
$ 
```


## Installation

1. Clone the repo
1. Install `cargo install --path .`
1. Use `snova`


## Configuration

The tool comes with built-in command definitions (`./defs/builtin.toml`). 
Additionally, it is possible to define custom commands in a `$HOME/.config/snova/commands.toml` file. 
Snova will try to find that file and include all commands from it.


## TODO:

* [x] Simple commands
  * [x] `find`
  * [x] `grep`
  * [x] Set git email address
  * [x] `curl`
* [x] Use alternative screen for building a command
* [x] Improve UI flow
* [x] Use bold and underline text for better contrast
* [x] Allow defining commands in a toml file
* [x] Read user commands from (`$HOME/.config/snova/commands.toml`)
* [x] Support enum value type (e.g curl methods GET/POST/..)
* [x] Support specifying value options 
    * If a field has free text you can suggest some commonly used values

* [ ] Better templating syntax. Atm it breaks frequently.
* [ ] Autocomplete and verify path value type
* [ ] Which other commands do I use?
* [ ] Set up clippy
* [ ] Pipe commands
* [ ] Killer feature: execute the same command over SSH


## Terminal libs

* https://lib.rs/crates/termion
* https://lib.rs/crates/crossterm
* https://lib.rs/crates/rustyline
* https://lib.rs/crates/console
