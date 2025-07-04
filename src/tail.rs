use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::time::Duration;

pub fn run(args: &[String]) -> i32 {
    let mut lines: Option<usize> = None;
    let mut bytes: Option<usize> = None;
    let mut follow = false;
    let mut paths: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-f" | "--follow" => follow = true,
            "-n" | "--lines" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("tail: option requires an argument -- 'n'");
                    return 1;
                }
                match args[i].parse::<usize>() {
                    Ok(n) => lines = Some(n),
                    Err(_) => {
                        eprintln!("tail: invalid number of lines: '{}'", args[i]);
                        return 1;
                    }
                }
            }
            "-c" | "--bytes" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("tail: option requires an argument -- 'c'");
                    return 1;
                }
                match args[i].parse::<usize>() {
                    Ok(n) => bytes = Some(n),
                    Err(_) => {
                        eprintln!("tail: invalid number of bytes: '{}'", args[i]);
                        return 1;
                    }
                }
            }
            arg if arg.starts_with("--lines=") => {
                match arg["--lines=".len()..].parse::<usize>() {
                    Ok(n) => lines = Some(n),
                    Err(_) => {
                        eprintln!("tail: invalid number of lines: '{}'", &arg["--lines=".len()..]);
                        return 1;
                    }
                }
            }
            arg if arg.starts_with("--bytes=") => {
                match arg["--bytes=".len()..].parse::<usize>() {
                    Ok(n) => bytes = Some(n),
                    Err(_) => {
                        eprintln!("tail: invalid number of bytes: '{}'", &arg["--bytes=".len()..]);
                        return 1;
                    }
                }
            }
            arg if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") => {
                let rest = &arg[1..];
                if rest.chars().all(|c| c.is_ascii_digit()) {
                    match rest.parse::<usize>() {
                        Ok(n) => lines = Some(n),
                        Err(_) => {
                            eprintln!("tail: invalid option: '{}'", arg);
                            return 1;
                        }
                    }
                } else {
                    let mut chars = rest.chars().peekable();
                    while let Some(ch) = chars.next() {
                        match ch {
                            'f' => follow = true,
                            'n' => {
                                let val: String = chars.collect();
                                let val = if val.is_empty() {
                                    i += 1;
                                    if i >= args.len() {
                                        eprintln!("tail: option requires an argument -- 'n'");
                                        return 1;
                                    }
                                    args[i].clone()
                                } else {
                                    val
                                };
                                match val.parse::<usize>() {
                                    Ok(n) => lines = Some(n),
                                    Err(_) => {
                                        eprintln!("tail: invalid number of lines: '{}'", val);
                                        return 1;
                                    }
                                }
                                break;
                            }
                            'c' => {
                                let val: String = chars.collect();
                                let val = if val.is_empty() {
                                    i += 1;
                                    if i >= args.len() {
                                        eprintln!("tail: option requires an argument -- 'c'");
                                        return 1;
                                    }
                                    args[i].clone()
                                } else {
                                    val
                                };
                                match val.parse::<usize>() {
                                    Ok(n) => bytes = Some(n),
                                    Err(_) => {
                                        eprintln!("tail: invalid number of bytes: '{}'", val);
                                        return 1;
                                    }
                                }
                                break;
                            }
                            _ => {
                                eprintln!("tail: invalid option -- '{}'", ch);
                                return 1;
                            }
                        }
                    }
                }
            }
            arg if arg.starts_with('-') => {
                eprintln!("tail: unrecognized option '{}'", arg);
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
        exit_code |= tail_reader(&mut io::stdin().lock(), &mut out, n_lines, bytes);
    } else {
        for (idx, path) in paths.iter().enumerate() {
            if multiple {
                if idx > 0 {
                    let _ = writeln!(out);
                }
                let _ = writeln!(out, "==> {} <==", path);
            }
            if path == "-" {
                exit_code |= tail_reader(&mut io::stdin().lock(), &mut out, n_lines, bytes);
            } else {
                match File::open(path) {
                    Ok(f) => {
                        let mut reader = BufReader::new(f);
                        exit_code |= tail_reader(&mut reader, &mut out, n_lines, bytes);
                    }
                    Err(e) => {
                        eprintln!("tail: cannot open '{}': {}", path, e);
                        exit_code = 1;
                    }
                }
            }
        }

        if follow && exit_code == 0 {
            let _ = out.flush();
            exit_code |= follow_files(&paths, n_lines, &mut out);
        }
    }

    exit_code
}

fn tail_reader<R: BufRead, W: Write>(
    reader: &mut R,
    out: &mut W,
    n_lines: usize,
    bytes: Option<usize>,
) -> i32 {
    if let Some(n_bytes) = bytes {
        let mut buf = Vec::new();
        if reader.read_to_end(&mut buf).is_err() {
            return 1;
        }
        let start = buf.len().saturating_sub(n_bytes);
        if out.write_all(&buf[start..]).is_err() {
            return 1;
        }
        return 0;
    }

    // Collect into a ring buffer of the last n_lines lines
    let mut ring: Vec<String> = Vec::with_capacity(n_lines + 1);
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                if ring.len() == n_lines {
                    ring.remove(0);
                }
                ring.push(line.clone());
            }
            Err(_) => return 1,
        }
    }

    for l in &ring {
        if out.write_all(l.as_bytes()).is_err() {
            return 1;
        }
    }

    0
}
