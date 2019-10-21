use clap::ArgMatches;
use std::env::current_dir;
use std::path::Path;
use std::error::Error;
use std::io::BufReader;
use std::fs::File;
use liquid_demons::{create_matrix, calculate, poll_result, Setting};
use serde_json::{json, Map, Value};
use crate::render::{pretty_print_result, pretty_print_settings};

pub const FILE: &str = "FILE";
pub const JSON: &str = "JSON";
pub const ALL: &str = "ALL";

pub fn run(matches: &ArgMatches) {
    let settings_file = matches.value_of(FILE).expect("please specify file");
    let path = current_dir().unwrap();
    let settings_path = path.join(settings_file);

    let settings = read_settings_from_file(&settings_path).expect("could not read file");
    // let voters = create_matrix_from_settings(settings);
    let m = create_matrix(&settings);
    
    let calulation_result = calculate(m, settings.voters.len());
    let result = poll_result(&settings.voters, &settings.policies, calulation_result);

    // printing the result
    if matches.is_present(JSON) {
        let mut out: Map<String, Value> = Map::new();
        if matches.is_present(ALL) {
            out.insert("input".to_string(), json!(settings));
        }
        out.insert("output".to_string(), json!(result));
        let result_json = serde_json::to_string_pretty(&out)
        .expect("could not convert result to JSON format");
        println!("{}", result_json);
    } else {

        if matches.is_present(ALL) {
            pretty_print_settings(&settings).unwrap();
        }
        pretty_print_result(&result).expect("something went wron when printing");
    }
}

fn read_settings_from_file(path: &Path) -> Result<Setting, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let settings: Setting = serde_json::from_reader(reader)?;
    Ok(settings)
}