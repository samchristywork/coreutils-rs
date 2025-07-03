use std::fs;
use std::io::{self, Read, Write};

use crate::term::{is_tty, Key, Term};

pub fn run(args: &[String]) -> i32 {
    let mut show_line_numbers = false;
    let mut paths: Vec<String> = Vec::new();

    for arg in args {
        if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") {
            for ch in arg[1..].chars() {
                match ch {
                    'N' => show_line_numbers = true,
                    _ => {
                        eprintln!("less: invalid option -- '{}'", ch);
                        return 1;
                    }
                }
            }
        } else {
            match arg.as_str() {
                "--LINE-NUMBERS" => show_line_numbers = true,
                a if a.starts_with('-') => {
                    eprintln!("less: unrecognized option '{}'", a);
                    return 1;
                }
                _ => paths.push(arg.clone()),
            }
        }
    }

    let content = if paths.is_empty() {
        let mut buf = String::new();
        if io::stdin().read_to_string(&mut buf).is_err() {
            eprintln!("less: error reading stdin");
            return 1;
        }
        buf
    } else {
        match fs::read_to_string(&paths[0]) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("less: {}: {}", paths[0], e);
                return 1;
            }
        }
    };

    let lines: Vec<&str> = content.split('\n').collect();
    let lines = if lines.last() == Some(&"") {
        &lines[..lines.len() - 1]
    } else {
        &lines[..]
    };

    if !is_tty() {
        let stdout = io::stdout();
        let mut out = io::BufWriter::new(stdout.lock());
        for line in lines {
            let _ = writeln!(out, "{}", line);
        }
        return 0;
    }

    let mut term = match Term::open() {
        Some(t) => t,
        None => {
            eprintln!("less: could not open terminal");
            return 1;
        }
    };
    term.enter_alt_screen();

    let mut pager = Pager {
        lines,
        top: 0,
        show_line_numbers,
        search: String::new(),
        search_matches: Vec::new(),
        term,
    };

    pager.run()
}
