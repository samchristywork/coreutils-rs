use std::fs::File;
use std::io::{self, BufReader, Read, Write};

pub fn run(args: &[String]) -> i32 {
    let mut silent = false;
    let mut verbose = false;
    let mut limit: Option<u64> = None;
    let mut skip1: u64 = 0;
    let mut skip2: u64 = 0;
    let mut paths: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            "-s" | "--silent" | "--quiet" => silent = true,
            "-l" | "--verbose" => verbose = true,
            "-n" | "--bytes" => {
                i += 1;
                if i >= args.len() { eprintln!("cmp: option requires an argument -- 'n'"); return 2; }
                match args[i].parse() {
                    Ok(n) => limit = Some(n),
                    Err(_) => { eprintln!("cmp: invalid byte count: '{}'", args[i]); return 2; }
                }
            }
            "-i" | "--ignore-initial" => {
                i += 1;
                if i >= args.len() { eprintln!("cmp: option requires an argument -- 'i'"); return 2; }
                match parse_skip(&args[i]) {
                    Some((a, b)) => { skip1 = a; skip2 = b; }
                    None => { eprintln!("cmp: invalid skip value: '{}'", args[i]); return 2; }
                }
            }
            _ if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") => {
                let mut chars = arg[1..].chars().peekable();
                while let Some(ch) = chars.next() {
                    match ch {
                        's' => silent = true,
                        'l' => verbose = true,
                        'n' | 'i' => {
                            let rest: String = chars.collect();
                            let val = if rest.is_empty() {
                                i += 1;
                                if i >= args.len() { eprintln!("cmp: option requires an argument -- '{}'", ch); return 2; }
                                args[i].clone()
                            } else { rest };
                            match ch {
                                'n' => match val.parse() {
                                    Ok(n) => limit = Some(n),
                                    Err(_) => { eprintln!("cmp: invalid byte count: '{}'", val); return 2; }
                                },
                                'i' => match parse_skip(&val) {
                                    Some((a, b)) => { skip1 = a; skip2 = b; }
                                    None => { eprintln!("cmp: invalid skip value: '{}'", val); return 2; }
                                },
                                _ => unreachable!(),
                            }
                            break;
                        }
                        _ => { eprintln!("cmp: invalid option -- '{}'", ch); return 2; }
                    }
                }
            }
            _ if arg.starts_with('-') => { eprintln!("cmp: unrecognized option '{}'", arg); return 2; }
            _ => paths.push(args[i].clone()),
        }
        i += 1;
    }

    if paths.len() < 2 {
        eprintln!("cmp: missing operand");
        return 2;
    }

    let mut r1 = match open(&paths[0]) {
        Ok(r) => r,
        Err(e) => { eprintln!("cmp: {}: {}", paths[0], e); return 2; }
    };
    let mut r2 = match open(&paths[1]) {
        Ok(r) => r,
        Err(e) => { eprintln!("cmp: {}: {}", paths[1], e); return 2; }
    };

    // Apply skips
    if skip1 > 0 && skip_bytes(&mut r1, skip1).is_err() { /* EOF before skip, continue */ }
    if skip2 > 0 && skip_bytes(&mut r2, skip2).is_err() { /* EOF before skip, continue */ }

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    let mut byte_num: u64 = 1;
    let mut line_num: u64 = 1;
    let mut diff_found = false;
    let mut buf1 = [0u8; 4096];
    let mut buf2 = [0u8; 4096];
    let mut pos1 = 0usize;
    let mut pos2 = 0usize;
    let mut len1 = 0usize;
    let mut len2 = 0usize;

    loop {
        if let Some(lim) = limit {
            if byte_num > lim { break; }
        }

        if pos1 >= len1 {
            len1 = r1.read(&mut buf1).unwrap_or(0);
            pos1 = 0;
        }
        if pos2 >= len2 {
            len2 = r2.read(&mut buf2).unwrap_or(0);
            pos2 = 0;
        }

        let eof1 = pos1 >= len1;
        let eof2 = pos2 >= len2;

        match (eof1, eof2) {
            (true, true) => break,
            (true, false) => {
                if !silent {
                    eprintln!("cmp: EOF on {} after byte {}, line {}", paths[0], byte_num - 1, line_num);
                }
                return 1;
            }
            (false, true) => {
                if !silent {
                    eprintln!("cmp: EOF on {} after byte {}, line {}", paths[1], byte_num - 1, line_num);
                }
                return 1;
            }
            (false, false) => {
                let b1 = buf1[pos1];
                let b2 = buf2[pos2];
                pos1 += 1;
                pos2 += 1;

                if b1 != b2 {
                    diff_found = true;
                    if verbose {
                        let _ = writeln!(out, "{:8} {:3o} {:3o}", byte_num, b1, b2);
                    } else if !silent {
                        println!("{} {} differ: byte {}, line {}", paths[0], paths[1], byte_num, line_num);
                        return 1;
                    } else {
                        return 1;
                    }
                }

                if b1 == b'\n' { line_num += 1; }
                byte_num += 1;
            }
        }
    }

    if diff_found { 1 } else { 0 }
}

fn open(path: &str) -> io::Result<Box<dyn Read>> {
    if path == "-" {
        Ok(Box::new(io::stdin().lock()))
    } else {
        Ok(Box::new(BufReader::new(File::open(path)?)))
    }
}

fn skip_bytes(r: &mut Box<dyn Read>, n: u64) -> io::Result<()> {
    let mut buf = [0u8; 4096];
    let mut remaining = n;
    while remaining > 0 {
        let to_read = remaining.min(buf.len() as u64) as usize;
        let got = r.read(&mut buf[..to_read])?;
        if got == 0 { break; }
        remaining -= got as u64;
    }
    Ok(())
}

fn parse_skip(s: &str) -> Option<(u64, u64)> {
    if let Some((a, b)) = s.split_once(':') {
        Some((a.parse().ok()?, b.parse().ok()?))
    } else {
        let n = s.parse().ok()?;
        Some((n, n))
    }
}
