mod handlers;
mod render;
use clap::{Arg, App, SubCommand};
use handlers::{run, ALL, FILE, JSON};

pub fn main () {

    let global_match = App::new("liq")
        .version("0.0.0")
        .about("a tool to support humans working collectivly to make descisions")
        .before_help(" _ _       
| (_) __ _ 
| | |/ _` |
| | | (_| |
|_|_|\\__, |
        |_|")
        .author("Yasushi Sakai")
        .subcommand(SubCommand::with_name("new")
            .about("starts a interactive liquid democracy session")
            .arg(Arg::with_name(FILE)
                .required(true)
            )
        )
        .subcommand(SubCommand::with_name("run")
            .arg(Arg::with_name(FILE)
                 .help("the configurated file to feed in liquid democracy calculation")
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
        ).get_matches();


    if let Some(run_matches) = global_match.subcommand_matches("run") {
        run(run_matches);
    }
}
