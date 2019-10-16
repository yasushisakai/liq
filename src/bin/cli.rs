
use clap::{Arg, App};
use std::env::current_dir;
use std::path::Path;
use std::error::Error;
use std::io::BufReader;
use std::fs::File;
use liquid_demons::{create_matrix, calculate, poll_result, Setting, PollResult};

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

    let settings = read_settings_from_file(&settings_path).expect("could not read file");
    // let voters = create_matrix_from_settings(settings);
    //
    let m = create_matrix(&settings);
    
    let calulation_result = calculate(m, settings.voters.len());
    let result = poll_result(&settings.voters, &settings.policies, calulation_result);

    pretty_print(&result);

}

fn pretty_print(result: &PollResult) {

    println!("*** result ***\n");

    println!("{:?}", result);

    println!("\n*** end result ***\n");

}

fn read_settings_from_file(path: &Path) -> Result<Setting, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let settings: Setting = serde_json::from_reader(reader)?;

    Ok(settings)
}
