use std::fs;
use std::io::{self, Read, Write};
use std::os::unix::io::AsRawFd;

use crate::term::{is_tty, read_fd, Key, Term};

pub fn run(args: &[String]) -> i32 {
    let mut paths: Vec<String> = Vec::new();

    for arg in args {
        if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") {
            for ch in arg[1..].chars() {
                match ch {
                    _ => {
                        eprintln!("more: invalid option -- '{}'", ch);
                        return 1;
                    }
                }
            }
        } else if arg.starts_with('-') && arg != "-" {
            eprintln!("more: unrecognized option '{}'", arg);
            return 1;
        } else {
            paths.push(arg.clone());
        }
    }

    let content = if paths.is_empty() {
        let mut buf = String::new();
        if io::stdin().read_to_string(&mut buf).is_err() {
            eprintln!("more: error reading stdin");
            return 1;
        }
        buf
    } else {
        let mut combined = String::new();
        for path in &paths {
            if path == "-" {
                if io::stdin().read_to_string(&mut combined).is_err() {
                    eprintln!("more: error reading stdin");
                    return 1;
                }
            } else {
                match fs::read_to_string(path) {
                    Ok(s) => combined.push_str(&s),
                    Err(e) => {
                        eprintln!("more: {}: {}", path, e);
                        return 1;
                    }
                }
            }
        }
        combined
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
            eprintln!("more: could not open terminal");
            return 1;
        }
    };

    page(&mut term, lines)
}
