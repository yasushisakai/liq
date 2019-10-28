use clap::ArgMatches;
use std::collections::HashMap;
use std::env::current_dir;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::io::{stdin, stdout, Read, Write};
use std::mem::replace;
use std::path::Path;
use std::thread;
use std::time;
// use std::io::prelude::*;
use crate::interactions::{ask_for_a_float, ask_for_a_string, chooser};
use crate::render::{pretty_print_result, pretty_print_settings};
use liq::{calculate, create_matrix, poll_result, Policy, Setting};
use serde_json::{json, Map, Value};
use termion::raw::IntoRawMode;
use termion::{clear, color, cursor, style};

pub const FILE: &str = "FILE";
pub const JSON: &str = "JSON";
pub const ALL: &str = "ALL";

pub fn run(matches: &ArgMatches) {
    let settings_file = matches.value_of(FILE).unwrap();
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
        let result_json =
            serde_json::to_string_pretty(&out).expect("could not convert result to JSON format");
        println!("{}", result_json);
    } else {
        if matches.is_present(ALL) {
            pretty_print_settings(&settings).unwrap();
        }
        pretty_print_result(&result).expect("something went wrong when printing");
    }
}

pub fn edit(matches: &ArgMatches) {
    println!("edit");
}

pub fn new(matches: &ArgMatches) {
    let mut settings = Setting::default();

    print!("{}Title (optional){}: ",
    style::Bold,
    style::Reset, 
    );
    stdout().flush().unwrap();

    let mut title = String::new();
    if stdin().read_line(&mut title).is_ok() {
        settings.title = Some(title.trim().to_string());
    }

    menu(&mut settings).expect("error in main menu");

    let path = current_dir().unwrap();
    let file_name = matches.value_of(FILE).unwrap();
    let file_path = path.join(Path::new(file_name));
    settings.purge_and_normalize();
    write_settings_file(&file_path, &settings).expect("couldn't write the file");
}

fn menu(settings: &mut Setting) -> Result<(), Box<dyn Error>> {
    let mut option: char = '0';
    {
        // Set terminal to raw mode to allow reading stdin one key at a time
        let mut stdout = stdout().into_raw_mode()?;
        // Use asynchronous stdin
        // let mut stdin = termion::async_stdin().keys();
        let stdin = stdin();
        let stdin = stdin.lock();

        // let options: HashMap<char, &str> = [('1', "Add voters"), ('2', "Delete voters"), ('3', "Rename voters")].iter().cloned().collect();

        writeln!(stdout,
        "{}Select Option{}:\n\r  (1) Edit voters\n\r  (2) Edit Policies\n\r  (3) Edit Votes\n\r  (q) Save and Quit\r",
        termion::style::Bold,
        termion::style::Reset
        )?;

        let mut bytes = stdin.bytes();

        loop {
            // let input = stdin.next();
            let byte = bytes.next();
            // If a key was pressed
            if let Some(Ok(key)) = byte {
                match key {
                    b'\n' | b'\r' => {
                        // ENTER
                        match option {
                            '1' | '2' | '3' | 'q' => {
                                writeln!(stdout, "\r")?;
                                // stdout.lock().flush()?;
                                break;
                            }
                            _ => {}
                        }
                    }
                    b => match b {
                        b'1' | b'2' | b'3' | b'q' => {
                            let c = b as char;
                            write!(
                                stdout,
                                "{}\r{}your choice:({}){}",
                                termion::clear::CurrentLine,
                                termion::style::Bold,
                                c,
                                termion::style::Reset
                            )?;
                            stdout.lock().flush()?;
                            option = c;
                        }
                        _ => {}
                    },
                }
                thread::sleep(time::Duration::from_millis(50));
            }
        }

        // clean the terminal
        for _ in 0..6 {
            write!(stdout, "{0}{1}\r", cursor::Up(1), clear::CurrentLine)?;
            stdout.lock().flush()?;
        }
    } // want to destroy raw mode stdout

    match option {
        '1' => edit_voters(settings).expect("error while editing voters"),
        '2' => edit_policies(settings).expect("error while editing policies"),
        '3' => edit_votes(settings).expect("error while editing votes"),
        _ => {}
    };

    Ok(())
}

fn edit_policies(settings: &mut Setting) -> Result<(), Box<dyn Error>> {
    let options: HashMap<char, &str> = [
        ('l', "list policies"),
        ('a', "Add policy"),
        ('d', "Delete policy"),
        ('e', "Edit Policy"),
    ]
    .iter()
    .cloned()
    .collect();

    let option = chooser(&options)?;

    let does_recuse = match option {
        'a' => {
            loop {
                if let Ok(short) = ask_for_a_string("id or short description:") {
                    loop {
                        if let Ok(long) = ask_for_a_string("long description(\"optional\"):") {
                            let p = if &long == "" {
                                Policy::Short(short)
                            } else {
                                Policy::Long((short, long))
                            };
                            settings.add_policy(p);
                            break;
                        }
                    }
                    break;
                };
            }
            true
        }
        'd' => {
            loop {
                if let Ok(short) = ask_for_a_string("id or short description to delete:") {
                    match settings.delete_policy(&short) {
                        Some(_) => break,
                        None => println!("could not find policy with id: {}", &short),
                    }
                }
            }
            true
        }
        'r' => {
            let mut flag = false;
            loop {
                if let Ok(old_short) = ask_for_a_string("policy to replace:") {
                    match settings.delete_voter(&old_short) {
                        Some(index) => loop {
                            if let Ok(new_short) =
                                ask_for_a_string("replace to (new id or short description):")
                            {
                                let p: Policy;
                                loop {
                                    if let Ok(new_long) =
                                        ask_for_a_string("description for new policy(optional):")
                                    {
                                        p = if &new_long == "" {
                                            Policy::Short(new_short)
                                        } else {
                                            Policy::Long((new_short, new_long))
                                        };
                                        break;
                                    }
                                }
                                replace(&mut settings.policies[index], p);
                                flag = true;
                                break;
                            }
                        },
                        None => println!("could not find policy id with: {}", &old_short),
                    }
                }
                if flag {
                    break;
                }
            }
            true
        }
        'l' => {
            for p in &settings.policies {
                match p {
                    Policy::Short(short) => println!("{}", short),
                    Policy::Long((id, desc)) => println!("{}: {}", id, desc),
                }
            }
            true
        }
        'b' => false,
        _ => false,
    };

    if does_recuse {
        edit_policies(settings)?;
    } else {
        menu(settings)?;
    }

    Ok(())
}

fn edit_voters(settings: &mut Setting) -> Result<(), Box<dyn Error>> {
    let options: HashMap<char, &str> = [
        ('l', "list view voters"),
        ('a', "Add voter"),
        ('d', "Delete voter"),
        ('r', "Rename voter"),
    ]
    .iter()
    .cloned()
    .collect();

    let option = chooser(&options)?;

    let does_recuse = match option {
        'a' => {
            // ask of name
            loop {
                if let Ok(name) = ask_for_a_string(&format!("{}New user name{}:", style::Bold, style::Reset)) {
                    settings.add_voter(&name);
                    break;
                };
            }
            true
        }
        'd' => {
            loop {
                if let Ok(name) = ask_for_a_string("delete user:") {
                    match settings.delete_voter(&name) {
                        Some(_) => break,
                        None => println!("could not find user name: {}", &name),
                    }
                }
            }
            true
        }
        'r' => {
            let mut flag = false;
            loop {
                if let Ok(old_name) = ask_for_a_string("user to remove:") {
                    match settings.delete_voter(&old_name) {
                        Some(index) => loop {
                            if let Ok(new_name) = ask_for_a_string("new user name:") {
                                replace(&mut settings.voters[index], new_name);
                                flag = true;
                                break;
                            }
                        },
                        None => println!("could not find user name: {}", &old_name),
                    }
                }
                if flag {
                    break;
                }
            }
            true
        }
        'l' => {
            for v in &settings.voters {
                println!("{}", v);
            }
            true
        }
        'b' => false,
        _ => false,
    };

    if does_recuse {
        edit_voters(settings)?;
    } else {
        menu(settings)?;
    }

    Ok(())
}

fn edit_votes(settings: &mut Setting) -> Result<(), Box<dyn Error>>{
    if let Ok(name) = ask_for_a_string("edit user name:") {
        match settings.voters.iter().position(|v| v==&name) {
            None => {
                println!("Could not find user name: {}", &name);
            },
            Some(_) => {
                println!("You can put any number to show your support for both fellow voters and policies. Note that ");
                println!(
                    "your votes will be normalized to be a total of {}one{} vote.",
                    style::Bold,
                    style::Reset
                );

                let votes = match settings.votes.get(&name) {
                    Some(vts) => vts.to_owned(),
                    None => HashMap::new()
                };
        
                println!("{}", &name);

                let mut new_vote: HashMap<String, f64> = HashMap::new();

                println!(
                    "{}{}Policies{}:",
                    style::Bold,
                    color::Fg(color::Green),
                    style::Reset
                );
                for policy in &settings.policies {
                    loop {
                        let mut current_value = 0.0;
                        let id = format!("{}", policy);
                        if let Some(v) = votes.get(&id) {
                            current_value = v.to_owned();
                        }
                        let policy_id = format!("{}", &policy);
                        if let Ok(v) = ask_for_a_float(&policy_id, current_value) {
                            new_vote.insert(policy_id, v);
                            break;
                        } 
                    }
                }

                println!(
                    "{}{}Voters{}:",
                    style::Bold,
                    color::Fg(color::Blue),
                    style::Reset
                );
                for voter in &settings.voters {
                    if &name == voter {
                        continue;
                    }
                    loop {
                        let mut current_value = 0.0;
                        if let Some(v) = votes.get(voter) {
                            current_value = v.to_owned();
                        }
                        if let Ok(v) = ask_for_a_float(&voter, current_value) {
                            new_vote.insert(voter.to_owned(), v);
                            break;
                        }
                    }
                }
                settings.votes.insert(name, new_vote);
            }
        }
    }
    // clean votes
    menu(settings)?;
    Ok(())
}

fn read_settings_from_file(path: &Path) -> Result<Setting, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let settings: Setting = serde_json::from_reader(reader)?;
    Ok(settings)
}

fn write_settings_file(file_path: &Path, settings: &Setting) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(file_path)?;
    let json_bytes = serde_json::to_vec_pretty(settings)?;
    file.write_all(&json_bytes)?;
    Ok(())
}
