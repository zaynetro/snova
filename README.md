# Snova

A CLI tool to help build you a command you forgot.

See it in action: [asciicast](https://asciinema.org/a/cCcRDmN1NXuoM8bL4IGVbEXlh)

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
* [ ] Autocomplete and verify path value type
* [ ] Support enum value type (e.g curl methods GET/POST/..)
* [ ] Support specifying value options 
    * If a field has free text you can suggest some commonly used values
* [ ] Set up clippy
* [ ] Pipe commands
* [ ] Killer feature: execute the same command over SSH


## Terminal libs

* https://lib.rs/crates/termion
* https://lib.rs/crates/crossterm
* https://lib.rs/crates/rustyline
* https://lib.rs/crates/console
