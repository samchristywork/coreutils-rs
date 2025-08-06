use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

pub fn run(args: &[String]) -> i32 {
    let mut separator = b'\n';
    let mut paths: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-s" | "--separator" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("tac: option requires an argument -- 's'");
                    return 1;
                }
                let s = args[i].as_bytes();
                if s.len() != 1 {
                    eprintln!("tac: separator must be a single byte");
                    return 1;
                }
                separator = s[0];
            }
            arg if arg.starts_with("--separator=") => {
                let s = &arg.as_bytes()["--separator=".len()..];
                if s.len() != 1 {
                    eprintln!("tac: separator must be a single byte");
                    return 1;
                }
                separator = s[0];
            }
            arg if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") => {
                if let Some(ch) = arg[1..].chars().next() {
                    match ch {
                        's' => {
                            i += 1;
                            if i >= args.len() {
                                eprintln!("tac: option requires an argument -- 's'");
                                return 1;
                            }
                            let s = args[i].as_bytes();
                            if s.len() != 1 {
                                eprintln!("tac: separator must be a single byte");
                                return 1;
                            }
                            separator = s[0];
                        }
                        _ => {
                            eprintln!("tac: invalid option -- '{}'", ch);
                            return 1;
                        }
                    }
                }
            }
            "-" => paths.push(args[i].clone()),
            arg if arg.starts_with('-') => {
                eprintln!("tac: unrecognized option '{}'", arg);
                return 1;
            }
            _ => paths.push(args[i].clone()),
        }
        i += 1;
    }

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let mut exit_code = 0;

    if paths.is_empty() {
        exit_code |= tac_reader(&mut io::stdin().lock(), &mut out, separator);
    } else {
        for path in &paths {
            if path == "-" {
                exit_code |= tac_reader(&mut io::stdin().lock(), &mut out, separator);
            } else {
                match File::open(path) {
                    Ok(f) => {
                        let mut reader = BufReader::new(f);
                        exit_code |= tac_reader(&mut reader, &mut out, separator);
                    }
                    Err(e) => {
                        eprintln!("tac: {}: {}", path, e);
                        exit_code = 1;
                    }
                }
            }
        }
    }

    exit_code
}

fn tac_reader<R: BufRead, W: Write>(reader: &mut R, out: &mut W, separator: u8) -> i32 {
    let mut records: Vec<Vec<u8>> = Vec::new();
    let mut record: Vec<u8> = Vec::new();

    let mut buf = [0u8; 65536];
    loop {
        match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                for &b in &buf[..n] {
                    record.push(b);
                    if b == separator {
                        records.push(record.clone());
                        record.clear();
                    }
                }
            }
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => {
                eprintln!("tac: read error: {}", e);
                return 1;
            }
        }
    }

    // Any trailing content without a terminating separator
    if !record.is_empty() {
        records.push(record);
    }

    for rec in records.iter().rev() {
        if out.write_all(rec).is_err() {
            return 1;
        }
    }

    0
}
