use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

pub fn run(args: &[String]) -> i32 {
    let mut count = false;
    let mut repeated = false;
    let mut unique = false;
    let mut ignore_case = false;
    let mut skip_fields: usize = 0;
    let mut skip_chars: usize = 0;
    let mut check_chars: Option<usize> = None;
    let mut paths: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            "-c" | "--count"          => count = true,
            "-d" | "--repeated"       => repeated = true,
            "-u" | "--unique"         => unique = true,
            "-i" | "--ignore-case"    => ignore_case = true,
            "-f" | "--skip-fields" => {
                i += 1;
                if i >= args.len() { eprintln!("uniq: option requires an argument -- 'f'"); return 1; }
                match args[i].parse() {
                    Ok(n) => skip_fields = n,
                    Err(_) => { eprintln!("uniq: invalid number of fields to skip: '{}'", args[i]); return 1; }
                }
            }
            "-s" | "--skip-chars" => {
                i += 1;
                if i >= args.len() { eprintln!("uniq: option requires an argument -- 's'"); return 1; }
                match args[i].parse() {
                    Ok(n) => skip_chars = n,
                    Err(_) => { eprintln!("uniq: invalid number of bytes to skip: '{}'", args[i]); return 1; }
                }
            }
            "-w" | "--check-chars" => {
                i += 1;
                if i >= args.len() { eprintln!("uniq: option requires an argument -- 'w'"); return 1; }
                match args[i].parse() {
                    Ok(n) => check_chars = Some(n),
                    Err(_) => { eprintln!("uniq: invalid number of bytes to compare: '{}'", args[i]); return 1; }
                }
            }
            _ if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") => {
                let mut chars = arg[1..].chars().peekable();
                while let Some(ch) = chars.next() {
                    match ch {
                        'c' => count = true,
                        'd' => repeated = true,
                        'u' => unique = true,
                        'i' => ignore_case = true,
                        'f' | 's' | 'w' => {
                            let rest: String = chars.collect();
                            let val = if rest.is_empty() {
                                i += 1;
                                if i >= args.len() {
                                    eprintln!("uniq: option requires an argument -- '{}'", ch);
                                    return 1;
                                }
                                args[i].clone()
                            } else { rest };
                            match ch {
                                'f' => match val.parse() {
                                    Ok(n) => skip_fields = n,
                                    Err(_) => { eprintln!("uniq: invalid number of fields: '{}'", val); return 1; }
                                },
                                's' => match val.parse() {
                                    Ok(n) => skip_chars = n,
                                    Err(_) => { eprintln!("uniq: invalid number of chars: '{}'", val); return 1; }
                                },
                                'w' => match val.parse() {
                                    Ok(n) => check_chars = Some(n),
                                    Err(_) => { eprintln!("uniq: invalid number of chars: '{}'", val); return 1; }
                                },
                                _ => unreachable!(),
                            }
                            break;
                        }
                        _ => { eprintln!("uniq: invalid option -- '{}'", ch); return 1; }
                    }
                }
            }
            _ if arg.starts_with('-') => { eprintln!("uniq: unrecognized option '{}'", arg); return 1; }
            _ => paths.push(args[i].clone()),
        }
        i += 1;
    }

    let input: Box<dyn BufRead> = match paths.first().map(|s| s.as_str()) {
        None | Some("-") => Box::new(io::stdin().lock()),
        Some(path) => match File::open(path) {
            Ok(f) => Box::new(BufReader::new(f)),
            Err(e) => { eprintln!("uniq: {}: {}", path, e); return 1; }
        }
    };

    let output_path = paths.get(1).cloned();
    let stdout;
    let file;
    let mut out: Box<dyn Write> = if let Some(ref p) = output_path {
        match File::create(p) {
            Ok(f) => { file = f; Box::new(io::BufWriter::new(&file)) }
            Err(e) => { eprintln!("uniq: {}: {}", p, e); return 1; }
        }
    } else {
        stdout = io::stdout();
        Box::new(io::BufWriter::new(stdout.lock()))
    };

    let opts = Opts { count, repeated, unique, ignore_case, skip_fields, skip_chars, check_chars };
    uniq_reader(input, &mut out, &opts)
}

struct Opts {
    count: bool,
    repeated: bool,
    unique: bool,
    ignore_case: bool,
    skip_fields: usize,
    skip_chars: usize,
    check_chars: Option<usize>,
}

fn uniq_reader<R: BufRead, W: Write>(mut reader: R, out: &mut W, opts: &Opts) -> i32 {
    let mut prev: Option<String> = None;
    let mut run_count: u64 = 0;
    let mut exit_code = 0;

    let mut line = String::new();
    loop {
        line.clear();
        let done = match reader.read_line(&mut line) {
            Ok(0) => true,
            Ok(_) => false,
            Err(_) => { exit_code = 1; true }
        };

        let current = if done {
            None
        } else {
            Some(line.trim_end_matches('\n').trim_end_matches('\r').to_string())
        };

        match (&prev, &current) {
            (Some(p), Some(c)) => {
                if compare_key(p, c, opts) {
                    run_count += 1;
                } else {
                    if should_print(run_count, opts) {
                        exit_code |= emit(p, run_count, opts, out);
                    }
                    prev = Some(c.clone());
                    run_count = 1;
                }
            }
            (None, Some(c)) => {
                prev = Some(c.clone());
                run_count = 1;
            }
            (Some(p), None) => {
                if should_print(run_count, opts) {
                    exit_code |= emit(p, run_count, opts, out);
                }
                break;
            }
            (None, None) => break,
        }

        if done { break; }
    }

    exit_code
}

fn compare_key(a: &str, b: &str, opts: &Opts) -> bool {
    let ka = extract_key(a, opts);
    let kb = extract_key(b, opts);
    if opts.ignore_case {
        ka.to_lowercase() == kb.to_lowercase()
    } else {
        ka == kb
    }
}

fn extract_key<'a>(s: &'a str, opts: &Opts) -> &'a str {
    let mut start = 0;

    if opts.skip_fields > 0 {
        let mut fields_skipped = 0;
        let mut in_space = true;
        for (i, ch) in s.char_indices() {
            if ch == ' ' || ch == '\t' {
                if !in_space {
                    fields_skipped += 1;
                    if fields_skipped == opts.skip_fields {
                        start = i;
                        break;
                    }
                    in_space = true;
                }
            } else {
                in_space = false;
            }
        }
        // Skip trailing whitespace after fields
        while start < s.len() && (s.as_bytes()[start] == b' ' || s.as_bytes()[start] == b'\t') {
            start += 1;
        }
    }

    let s = &s[start..];
    let s = if opts.skip_chars > 0 {
        let byte_pos = s.char_indices().nth(opts.skip_chars).map(|(i, _)| i).unwrap_or(s.len());
        &s[byte_pos..]
    } else {
        s
    };

    if let Some(n) = opts.check_chars {
        let byte_end = s.char_indices().nth(n).map(|(i, _)| i).unwrap_or(s.len());
        &s[..byte_end]
    } else {
        s
    }
}

fn should_print(count: u64, opts: &Opts) -> bool {
    if opts.repeated {
        count > 1
    } else if opts.unique {
        count == 1
    } else {
        true
    }
}

fn emit<W: Write>(line: &str, count: u64, opts: &Opts, out: &mut W) -> i32 {
    if opts.count {
        if writeln!(out, "{:7} {}", count, line).is_err() { return 1; }
    } else if writeln!(out, "{}", line).is_err() { return 1; }
    0
}
