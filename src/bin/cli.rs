use clap::{Arg, App, SubCommand};
use std::env::current_dir;
use std::path::Path;
use std::error::Error;
use std::io::BufReader;
use std::fs::File;
use std::iter::FromIterator;
use std::cmp::Ordering;
use liquid_demons::{create_matrix, calculate, poll_result, Setting, PollResult};
use serde_json::{json, Map, Value};
use std::io::prelude::*;
use term;
use term::StdoutTerminal;

pub fn main () {

    let matches = App::new("liq")
        .version("0.0.0")
        .about("liquid demons")
        .author("Yasushi Sakai")
        .subcommand(SubCommand::with_name("new")
            .about("starts a interactive liquid democracy session")
        )
        .arg(Arg::with_name("file")
             .short("f")
             .value_name("FILE")
             .help("Reads a setting file, and prints out the result")
        )
        .arg(Arg::with_name("json")
            .short("j")
            .takes_value(false)
            .help("prints result as json file")
        )
        .arg(Arg::with_name("all")
             .short("a")
             .takes_value(false)
             .help("prints the input setting and the result")
        )
        .get_matches();

    let settings_file = matches.value_of("file").expect("please specify file");
    let path = current_dir().unwrap();
    let settings_path = path.join(settings_file);

    let settings = read_settings_from_file(&settings_path).expect("could not read file");
    // let voters = create_matrix_from_settings(settings);
    let m = create_matrix(&settings);
    
    let calulation_result = calculate(m, settings.voters.len());
    let result = poll_result(&settings.voters, &settings.policies, calulation_result);

    // printing the result

    if matches.is_present("json") {

        let mut out: Map<String, Value> = Map::new();

        if matches.is_present("all") {
            out.insert("input".to_string(), json!(settings));
        }

        out.insert("output".to_string(), json!(result));

        let result_json = serde_json::to_string_pretty(&out)
        .expect("could not convert result to JSON format");
        println!("{}", result_json);
    } else {

        if matches.is_present("all") {
            pretty_print_settings(&settings).unwrap();
        }
        pretty_print_result(&result).expect("something went wron when printing");
    }
}

fn pretty_print_result(result: &PollResult) -> Result<(), Box<dyn Error>> {

    let mut t = term::stdout().unwrap();

    // sort the results
    let mut votes = Vec::from_iter(&result.votes);
    votes.sort_by(|&(_, a), &(_, b)| b.partial_cmp(&a).unwrap_or(Ordering::Equal));
    let mut influence = Vec::from_iter(&result.influence);
    influence.sort_by(|&(_, a), &(_, b)| b.partial_cmp(&a).unwrap_or(Ordering::Equal));

    t.fg(term::color::GREEN)?;
    t.attr(term::Attr::Bold)?;
    // t.fg(term::color::WHITE).unwrap();
    t.reset()?;
    for (p, v) in &votes {
        t.attr(term::Attr::Bold)?;
        t.fg(term::color::GREEN)?;
        write!(t, "{}", p)?;
        t.reset()?;
        write!(t, "\t{:.4}\n", v)?;
    }

    write!(t, "\r\n")?;

    t.fg(term::color::BLUE)?;
    t.attr(term::Attr::Bold)?;
    t.reset()?;
    for (inf, v) in &influence {
        t.attr(term::Attr::Bold)?;
        t.fg(term::color::BLUE)?;
        write!(t, "{}", inf)?;
        t.reset()?;
        write!(t, "\t{:.4}\n", v)?;
    }

    Ok(())
}

fn pretty_print_settings(setting: &Setting) -> Result<(), Box<dyn Error>> {

    let mut t = term::stdout().unwrap();

    t.attr(term::Attr::Bold)?;
    write!(t, "title ")?;
    t.reset()?;

    if let Some(title) = &setting.title {
        write!(t, "{}\n", title)?;
    }

    Ok(())
}

fn read_settings_from_file(path: &Path) -> Result<Setting, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let settings: Setting = serde_json::from_reader(reader)?;

    Ok(settings)
}
