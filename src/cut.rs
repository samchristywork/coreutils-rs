use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

pub fn run(args: &[String]) -> i32 {
    let mut fields: Option<Vec<Range>> = None;
    let mut bytes: Option<Vec<Range>> = None;
    let mut chars: Option<Vec<Range>> = None;
    let mut delimiter = b'\t';
    let mut only_delimited = false;
    let mut paths: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            "-f" | "--fields" => {
                i += 1;
                if i >= args.len() { eprintln!("cut: option requires an argument -- 'f'"); return 1; }
                match parse_ranges(&args[i]) {
                    Some(r) => fields = Some(r),
                    None => { eprintln!("cut: invalid field list"); return 1; }
                }
            }
            "-b" | "--bytes" => {
                i += 1;
                if i >= args.len() { eprintln!("cut: option requires an argument -- 'b'"); return 1; }
                match parse_ranges(&args[i]) {
                    Some(r) => bytes = Some(r),
                    None => { eprintln!("cut: invalid byte list"); return 1; }
                }
            }
            "-c" | "--characters" => {
                i += 1;
                if i >= args.len() { eprintln!("cut: option requires an argument -- 'c'"); return 1; }
                match parse_ranges(&args[i]) {
                    Some(r) => chars = Some(r),
                    None => { eprintln!("cut: invalid character list"); return 1; }
                }
            }
            "-d" | "--delimiter" => {
                i += 1;
                if i >= args.len() { eprintln!("cut: option requires an argument -- 'd'"); return 1; }
                let b = args[i].as_bytes();
                if b.len() != 1 {
                    eprintln!("cut: the delimiter must be a single character");
                    return 1;
                }
                delimiter = b[0];
            }
            "-s" | "--only-delimited" => only_delimited = true,
            _ if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") => {
                let mut chars_iter = arg[1..].chars().peekable();
                while let Some(ch) = chars_iter.next() {
                    match ch {
                        's' => only_delimited = true,
                        'f' | 'b' | 'c' | 'd' => {
                            let rest: String = chars_iter.collect();
                            let val = if rest.is_empty() {
                                i += 1;
                                if i >= args.len() {
                                    eprintln!("cut: option requires an argument -- '{}'", ch);
                                    return 1;
                                }
                                args[i].clone()
                            } else {
                                rest
                            };
                            match ch {
                                'f' => match parse_ranges(&val) {
                                    Some(r) => fields = Some(r),
                                    None => { eprintln!("cut: invalid field list"); return 1; }
                                },
                                'b' => match parse_ranges(&val) {
                                    Some(r) => bytes = Some(r),
                                    None => { eprintln!("cut: invalid byte list"); return 1; }
                                },
                                'c' => match parse_ranges(&val) {
                                    Some(r) => chars = Some(r),
                                    None => { eprintln!("cut: invalid character list"); return 1; }
                                },
                                'd' => {
                                    let b = val.as_bytes();
                                    if b.len() != 1 {
                                        eprintln!("cut: the delimiter must be a single character");
                                        return 1;
                                    }
                                    delimiter = b[0];
                                }
                                _ => unreachable!(),
                            }
                            break;
                        }
                        _ => {
                            eprintln!("cut: invalid option -- '{}'", ch);
                            return 1;
                        }
                    }
                }
            }
            _ if arg.starts_with('-') => {
                eprintln!("cut: unrecognized option '{}'", arg);
                return 1;
            }
            _ => paths.push(args[i].clone()),
        }
        i += 1;
    }

    if fields.is_none() && bytes.is_none() && chars.is_none() {
        eprintln!("cut: you must specify a list of bytes, characters, or fields");
        return 1;
    }

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let mut exit_code = 0;

    if paths.is_empty() {
        exit_code |= cut_reader(&mut io::stdin().lock(), &mut out, &fields, &bytes, &chars, delimiter, only_delimited);
    } else {
        for path in &paths {
            if path == "-" {
                exit_code |= cut_reader(&mut io::stdin().lock(), &mut out, &fields, &bytes, &chars, delimiter, only_delimited);
            } else {
                match File::open(path) {
                    Ok(f) => {
                        exit_code |= cut_reader(&mut BufReader::new(f), &mut out, &fields, &bytes, &chars, delimiter, only_delimited);
                    }
                    Err(e) => {
                        eprintln!("cut: {}: {}", path, e);
                        exit_code = 1;
                    }
                }
            }
        }
    }

    exit_code
}

fn cut_reader<R: BufRead, W: Write>(
    reader: &mut R,
    out: &mut W,
    fields: &Option<Vec<Range>>,
    bytes: &Option<Vec<Range>>,
    chars: &Option<Vec<Range>>,
    delimiter: u8,
    only_delimited: bool,
) -> i32 {
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(_) => return 1,
        }
        let content = line.trim_end_matches('\n').trim_end_matches('\r');

        if let Some(ranges) = fields {
            let has_delim = content.as_bytes().contains(&delimiter);
            if !has_delim {
                if !only_delimited {
                    let _ = writeln!(out, "{}", content);
                }
                continue;
            }
            let parts: Vec<&str> = content.split(delimiter as char).collect();
            let selected: Vec<&str> = select_indices(&parts, ranges);
            let _ = writeln!(out, "{}", selected.join(&(delimiter as char).to_string()));
        } else if let Some(ranges) = bytes {
            let b = content.as_bytes();
            let selected: Vec<u8> = select_indices(&b.iter().copied().collect::<Vec<u8>>(), ranges);
            let _ = out.write_all(&selected);
            let _ = writeln!(out);
        } else if let Some(ranges) = chars {
            let ch_vec: Vec<char> = content.chars().collect();
            let selected: Vec<char> = select_indices(&ch_vec, ranges);
            let s: String = selected.into_iter().collect();
            let _ = writeln!(out, "{}", s);
        }
    }
    0
}
