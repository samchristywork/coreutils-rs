use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

pub fn run(args: &[String]) -> i32 {
    let mut delimiter = vec![b'\t'];
    let mut serial = false;
    let mut paths: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            "-s" | "--serial" => serial = true,
            "-d" | "--delimiters" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("paste: option requires an argument -- 'd'");
                    return 1;
                }
                delimiter = parse_delimiters(&args[i]);
            }
            _ if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") => {
                let mut chars = arg[1..].chars().peekable();
                while let Some(ch) = chars.next() {
                    match ch {
                        's' => serial = true,
                        'd' => {
                            let rest: String = chars.collect();
                            let val = if rest.is_empty() {
                                i += 1;
                                if i >= args.len() {
                                    eprintln!("paste: option requires an argument -- 'd'");
                                    return 1;
                                }
                                args[i].clone()
                            } else {
                                rest
                            };
                            delimiter = parse_delimiters(&val);
                            break;
                        }
                        _ => {
                            eprintln!("paste: invalid option -- '{}'", ch);
                            return 1;
                        }
                    }
                }
            }
            _ if arg.starts_with('-') => {
                eprintln!("paste: unrecognized option '{}'", arg);
                return 1;
            }
            _ => paths.push(args[i].clone()),
        }
        i += 1;
    }

    if paths.is_empty() {
        paths.push("-".to_string());
    }

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    if serial {
        paste_serial(&paths, &delimiter, &mut out)
    } else {
        paste_parallel(&paths, &delimiter, &mut out)
    }
}

fn paste_parallel<W: Write>(paths: &[String], delimiters: &[u8], out: &mut W) -> i32 {
    let mut readers: Vec<Box<dyn BufRead>> = Vec::new();
    for path in paths {
        match open(path) {
            Some(r) => readers.push(r),
            None => return 1,
        }
    }

    let mut exit_code = 0;
    let mut line = String::new();

    loop {
        let mut any = false;
        let mut row = Vec::new();

        for reader in readers.iter_mut() {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => row.push(String::new()),
                Ok(_) => {
                    any = true;
                    let trimmed = line.trim_end_matches('\n').trim_end_matches('\r');
                    row.push(trimmed.to_string());
                }
                Err(_) => {
                    exit_code = 1;
                    row.push(String::new());
                }
            }
        }

        if !any {
            break;
        }

        let mut output = String::new();
        for (i, field) in row.iter().enumerate() {
            output.push_str(field);
            if i + 1 < row.len() {
                let d = delimiters[i % delimiters.len()];
                if d != 0 {
                    output.push(d as char);
                }
            }
        }
        let _ = writeln!(out, "{}", output);
    }

    exit_code
}

fn paste_serial<W: Write>(paths: &[String], delimiters: &[u8], out: &mut W) -> i32 {
    let mut exit_code = 0;

    for path in paths {
        let mut reader = match open(path) {
            Some(r) => r,
            None => return 1,
        };

        let mut line = String::new();
        let mut first = true;
        let mut del_idx = 0;

        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {}
                Err(_) => {
                    exit_code = 1;
                    break;
                }
            }

            let content = line.trim_end_matches('\n').trim_end_matches('\r');

            if !first {
                let d = delimiters[del_idx % delimiters.len()];
                del_idx += 1;
                if d != 0 {
                    let _ = write!(out, "{}", d as char);
                }
            }
            let _ = write!(out, "{}", content);
            first = false;
        }

        let _ = writeln!(out);
    }

    exit_code
}

fn open(path: &str) -> Option<Box<dyn BufRead>> {
    if path == "-" {
        Some(Box::new(io::stdin().lock()))
    } else {
        match File::open(path) {
            Ok(f) => Some(Box::new(BufReader::new(f))),
            Err(e) => {
                eprintln!("paste: {}: {}", path, e);
                None
            }
        }
    }
}

fn parse_delimiters(s: &str) -> Vec<u8> {
    let mut out = Vec::new();
    let mut chars = s.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('n') => out.push(b'\n'),
                Some('t') => out.push(b'\t'),
                Some('0') => out.push(0), // NUL = no delimiter
                Some('\\') => out.push(b'\\'),
                Some(c) => { out.push(b'\\'); out.push(c as u8); }
                None => out.push(b'\\'),
            }
        } else {
            out.push(ch as u8);
        }
    }
    if out.is_empty() { vec![b'\t'] } else { out }
}
