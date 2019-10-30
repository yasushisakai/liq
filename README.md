```
 _ _       
| (_) __ _ 
| | |/ _` |
| | | (_| |
|_|_|\__, |
        |_|

liq 0.1.0
Yasushi Sakai
cli tool to support humans working collectively to make decisions

USAGE:
    liq [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    new     starts a interactive liquid democracy session
    run     runs liquid democracy analysis from json file
    edit    edits the config file
    help    Prints this message or the help of the given subcommand(s)
```

[![asciicast](https://asciinema.org/a/277585.svg)](https://asciinema.org/a/277585)

# How to Build

1. install [rust](https://www.rust-lang.org/tools/install)
2. clone this repo
   ```
   git clone https://github.com/yasushisakai/liq
   ```
3. build
   ```
   cd liq
   cargo build --bin liq 
   ```
    this will make a binary called liq inside `target/release`
4. help humans make collective decisions !
   ```
   ./liq help
   ```