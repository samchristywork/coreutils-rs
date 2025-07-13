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
