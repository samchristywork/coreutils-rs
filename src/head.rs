use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

pub fn run(args: &[String]) -> i32 {
    let mut lines: Option<usize> = None;
    let mut bytes: Option<usize> = None;
    let mut paths: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-n" | "--lines" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("head: option requires an argument -- 'n'");
                    return 1;
                }
                match parse_count(&args[i]) {
                    Some(n) => lines = Some(n),
                    None => {
                        eprintln!("head: invalid number of lines: '{}'", args[i]);
                        return 1;
                    }
                }
            }
            "-c" | "--bytes" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("head: option requires an argument -- 'c'");
                    return 1;
                }
                match parse_count(&args[i]) {
                    Some(n) => bytes = Some(n),
                    None => {
                        eprintln!("head: invalid number of bytes: '{}'", args[i]);
                        return 1;
                    }
                }
            }
            arg if arg.starts_with("--lines=") => {
                match parse_count(&arg["--lines=".len()..]) {
                    Some(n) => lines = Some(n),
                    None => {
                        eprintln!("head: invalid number of lines: '{}'", &arg["--lines=".len()..]);
                        return 1;
                    }
                }
            }
            arg if arg.starts_with("--bytes=") => {
                match parse_count(&arg["--bytes=".len()..]) {
                    Some(n) => bytes = Some(n),
                    None => {
                        eprintln!("head: invalid number of bytes: '{}'", &arg["--bytes=".len()..]);
                        return 1;
                    }
                }
            }
            arg if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") => {
                // Support -N shorthand (e.g. -20)
                let rest = &arg[1..];
                if rest.chars().all(|c| c.is_ascii_digit()) {
                    match rest.parse::<usize>() {
                        Ok(n) => lines = Some(n),
                        Err(_) => {
                            eprintln!("head: invalid option: '{}'", arg);
                            return 1;
                        }
                    }
                } else {
                    // multi-flag like -n20 or -c20
                    let mut chars = rest.chars();
                    match chars.next() {
                        Some('n') => {
                            let val: String = chars.collect();
                            let val = if val.is_empty() {
                                i += 1;
                                if i >= args.len() {
                                    eprintln!("head: option requires an argument -- 'n'");
                                    return 1;
                                }
                                args[i].clone()
                            } else {
                                val
                            };
                            match parse_count(&val) {
                                Some(n) => lines = Some(n),
                                None => {
                                    eprintln!("head: invalid number of lines: '{}'", val);
                                    return 1;
                                }
                            }
                        }
                        Some('c') => {
                            let val: String = chars.collect();
                            let val = if val.is_empty() {
                                i += 1;
                                if i >= args.len() {
                                    eprintln!("head: option requires an argument -- 'c'");
                                    return 1;
                                }
                                args[i].clone()
                            } else {
                                val
                            };
                            match parse_count(&val) {
                                Some(n) => bytes = Some(n),
                                None => {
                                    eprintln!("head: invalid number of bytes: '{}'", val);
                                    return 1;
                                }
                            }
                        }
                        _ => {
                            eprintln!("head: invalid option: '{}'", arg);
                            return 1;
                        }
                    }
                }
            }
            arg if arg.starts_with('-') => {
                eprintln!("head: unrecognized option '{}'", arg);
                return 1;
            }
            _ => paths.push(args[i].clone()),
        }
        i += 1;
    }

    let n_lines = lines.unwrap_or(10);
    let multiple = paths.len() > 1;
    let mut exit_code = 0;
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    if paths.is_empty() {
        exit_code |= head_reader(&mut io::stdin().lock(), &mut out, n_lines, bytes);
    } else {
        for (idx, path) in paths.iter().enumerate() {
            if multiple {
                if idx > 0 {
                    let _ = writeln!(out);
                }
                let _ = writeln!(out, "==> {} <==", path);
            }
            if path == "-" {
                exit_code |= head_reader(&mut io::stdin().lock(), &mut out, n_lines, bytes);
            } else {
                match File::open(path) {
                    Ok(f) => {
                        let mut reader = BufReader::new(f);
                        exit_code |= head_reader(&mut reader, &mut out, n_lines, bytes);
                    }
                    Err(e) => {
                        eprintln!("head: cannot open '{}': {}", path, e);
                        exit_code = 1;
                    }
                }
            }
        }
    }

    exit_code
}
