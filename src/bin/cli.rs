
use clap::{Arg, App};
use std::env::current_dir;
use std::path::Path;
use std::error::Error;
use std::io::BufReader;
use std::fs::File;
use serde_json::Value;
use liquid_demons::{create_matrix, calculate, Setting};

pub fn main () {

    let matches = App::new("liq")
        .version("0.0.0")
        .about("liquid demons")
        .author("Yasushi Sakai")
        .arg(Arg::with_name("file")
             .short("f")
             .value_name("FILE")
             .help("Reads a setting file, and prints out the result")
        )
        .get_matches();

    let settings_file = matches.value_of("file").expect("please specify file");
    let path = current_dir().unwrap();
    let settings_path = path.join(settings_file);

    println!("{}", &settings_path.display());

    let settings = read_settings_from_file(&settings_path).unwrap();
    // let voters = create_matrix_from_settings(settings);
    //
    let m = create_matrix(&settings);
    
    let result = calculate(m, settings.voters.len());

    pretty_print(result, &settings);

}

fn pretty_print(result: (Vec<f64>, Vec<f64>), settings: &Setting) {

    let (poll, influence) = result;

    println!("*** result ***");

    println!("\npolices/plans:");

    for (i, r) in poll.iter().enumerate() {
       println!("{}:\t{}", &settings.policies.get(i).unwrap(), r);
    }

    println!("\ninfluence:");

    for (i, v) in influence.iter().enumerate() {
        println!("{}:\t{}", &settings.voters.get(i).unwrap(), v)
    }

    println!("\n*** end result ***\n");

}

fn read_settings_from_file(path: &Path) -> Result<Setting, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let settings: Setting = serde_json::from_reader(reader)?;

    Ok(settings)
}
