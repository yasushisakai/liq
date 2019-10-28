mod handlers;
mod render;
mod interactions;
use clap::{Arg, App, SubCommand};
use handlers::{edit, new, run, ALL, FILE, JSON};

pub fn main () {

    let global_match = App::new("liq")
        .version("0.1.0")
        .about("cli tool to support humans working collectively to make decisions")
        .before_help(" _ _       
| (_) __ _ 
| | |/ _` |
| | | (_| |
|_|_|\\__, |
        |_|")
        .author("Yasushi Sakai")
        .subcommand(SubCommand::with_name("new")
            .about("starts a interactive liquid democracy session")
            .display_order(1)
            .arg(Arg::with_name(FILE)
                .required(true)
            )
        )
        .subcommand(SubCommand::with_name("run")
            .about("runs liquid democracy analysis from json file")
            .display_order(2)
            .arg(Arg::with_name(FILE)
                 .help("the configured file to feed in liquid democracy calculation")
                 .required(true)
            )
            .arg(Arg::with_name(JSON)
                .short("j")
                .long("json")
                .help("prints result as json file")
            )
            .arg(Arg::with_name(ALL)
                 .short("a")
                 .long("all")
                 .help("prints the input setting and the result")
            )
        )
        .subcommand(SubCommand::with_name("edit")
            .about("edits the config file")
            .display_order(3)
            .arg(Arg::with_name(FILE)
                .help("the configuration file to edit")
                .required(true)
            )
        ).get_matches();

    match global_match.subcommand() {
        ("run", Some(m)) => run(m),
        ("new", Some(m)) => new(m),
        ("edit", Some(m)) => edit(m),
        _ => {}
    }
}
