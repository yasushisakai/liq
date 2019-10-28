use std::io::{stdin,stdout,Read,Write};
use std::error::Error;
use std::collections::HashMap;
use termion::raw::IntoRawMode;
use termion::{cursor, clear, style};
use std::thread;
use std::time;

pub fn chooser(options: &HashMap<char, &str>) -> Result<char, Box<dyn Error>> {
    // Set terminal to raw mode to allow reading stdin one key at a time
    let mut stdout = stdout().into_raw_mode()?;
    let stdin = stdin();
    let stdin = stdin.lock();

    writeln!(
        stdout,
        "{}Select Option{}:\r",
        style::Bold,
        style::Reset
    )?;
    for (c, mes) in options {
        writeln!(stdout, "  ({}) {}\r", c, mes)?;
    }
    writeln!(stdout, "  (b) Back\r")?;

    let mut option: char = '\0'; // null char here

    let mut bytes = stdin.bytes();
    loop {
        let byte = bytes.next();
        if let Some(Ok(key)) = byte {
            match key {
                b'\n' | b'\r' => {
                    if option != '\0' {
                        writeln!(stdout, "\n")?;
                        // stdout.lock().flush()?;
                        break;
                    }
                }
                b'b' => {
                    write!(stdout, "{}\ryour choice: (b): go Back", clear::CurrentLine)?;
                    stdout.lock().flush()?;
                    option = 'b';
                }
                b => {
                    let c = b as char;
                    if let Some(os) = options.get(&c) {
                        write!(
                            stdout,
                            "{}\ryour choice: ({}): {}",
                            clear::CurrentLine,
                            c,
                            os,
                        )?;
                        option = c;
                    }
                    stdout.lock().flush()?;
                }
            }
            thread::sleep(time::Duration::from_millis(50));
        }
    }
    // clean the terminal
    for _ in 0..options.len() + 4 {
        write!(stdout, "{0}{1}\r", cursor::Up(1), clear::CurrentLine)?;
        stdout.lock().flush()?;
    }

        if let Some(o) = options.get(&option) {
            println!("{}Selected{}: {}\r",
            style::Bold,
            style::Reset,
            o 
            );
        }

    Ok(option)
}

pub fn ask_for_a_string(mes: &str) -> Result<String, Box<dyn Error>> {
    print!("{} ", mes);
    stdout().flush()?;
    let mut out = String::new();
    stdin().read_line(&mut out)?;
    Ok(out.trim().to_string())
}

pub fn ask_for_a_float(mes: &str, default_value: f64) -> Result<f64, Box<dyn Error>> {
    let message = format!("{}({}): ", mes, default_value);
    let string_result = ask_for_a_string(&message)?;
    if string_result == "" {
        return Ok(default_value);
    }
    let r: f64 = string_result.parse()?;
    Ok(r)
}