# What was that command again? (wwtca) aka snova

## TODO:

* [ ] Simple commands
  * [x] Find files or directories `find PATH EXPRESSION` Where expression:
      * `-name PATTERN` File name pattern
  * [x] Find lines in a file`grep [OPTION...] PATTERNS FILE` Where options:
      * `-i` Case insensitive matching
      * `-v` Invert match (return non-matching lines)
      * `-n` Include line numbers
      * `-A NUM` Print NUM lines after the matched line
      * `-B NUM` Print NUM lines before matched line
      * `-r` Search files recursively
  * [x] Set git email address `git config [--global] user.email "EMAIL"`
  * [ ] awk
* [ ] Allow defining commands in a toml file
* [ ] Use bold and underline text for better contrast
* [ ] Pipe commands
* [ ] Killer feature: execute the same command over SSH


## Terminal libs

* https://lib.rs/crates/termion
* https://lib.rs/crates/crossterm
* https://lib.rs/crates/rustyline
* https://lib.rs/crates/console
