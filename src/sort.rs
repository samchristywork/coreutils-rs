use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

pub fn run(args: &[String]) -> i32 {
    let mut numeric = false;
    let mut reverse = false;
    let mut unique = false;
    let mut ignore_case = false;
    let mut human_numeric = false;
    let mut month_sort = false;
    let mut random_sort = false;
    let mut stable = false;
    let mut check = false;
    let mut check_quiet = false;
    let mut field_sep: Option<u8> = None;
    let mut keys: Vec<Key> = Vec::new();
    let mut output: Option<String> = None;
    let mut paths: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            "-n" | "--numeric-sort"       => numeric = true,
            "-r" | "--reverse"            => reverse = true,
            "-u" | "--unique"             => unique = true,
            "-f" | "--ignore-case"        => ignore_case = true,
            "-h" | "--human-numeric-sort" => human_numeric = true,
            "-M" | "--month-sort"         => month_sort = true,
            "-R" | "--random-sort"        => random_sort = true,
            "-s" | "--stable"             => stable = true,
            "-c" | "--check"              => check = true,
            "-C"                          => check_quiet = true,
            "-t" | "--field-separator" => {
                i += 1;
                if i >= args.len() { eprintln!("sort: option requires an argument -- 't'"); return 1; }
                let b = args[i].as_bytes();
                if b.len() != 1 { eprintln!("sort: multi-character tab is not allowed"); return 1; }
                field_sep = Some(b[0]);
            }
            "-k" | "--key" => {
                i += 1;
                if i >= args.len() { eprintln!("sort: option requires an argument -- 'k'"); return 1; }
                match parse_key(&args[i]) {
                    Some(k) => keys.push(k),
                    None => { eprintln!("sort: invalid key spec '{}'", args[i]); return 1; }
                }
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() { eprintln!("sort: option requires an argument -- 'o'"); return 1; }
                output = Some(args[i].clone());
            }
            _ if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") => {
                let mut chars = arg[1..].chars().peekable();
                while let Some(ch) = chars.next() {
                    match ch {
                        'n' => numeric = true,
                        'r' => reverse = true,
                        'u' => unique = true,
                        'f' => ignore_case = true,
                        'h' => human_numeric = true,
                        'M' => month_sort = true,
                        'R' => random_sort = true,
                        's' => stable = true,
                        'c' => check = true,
                        'C' => check_quiet = true,
                        't' | 'k' | 'o' => {
                            let rest: String = chars.collect();
                            let val = if rest.is_empty() {
                                i += 1;
                                if i >= args.len() {
                                    eprintln!("sort: option requires an argument -- '{}'", ch);
                                    return 1;
                                }
                                args[i].clone()
                            } else { rest };
                            match ch {
                                't' => {
                                    let b = val.as_bytes();
                                    if b.len() != 1 { eprintln!("sort: multi-character tab is not allowed"); return 1; }
                                    field_sep = Some(b[0]);
                                }
                                'k' => match parse_key(&val) {
                                    Some(k) => keys.push(k),
                                    None => { eprintln!("sort: invalid key spec '{}'", val); return 1; }
                                },
                                'o' => output = Some(val),
                                _ => unreachable!(),
                            }
                            break;
                        }
                        _ => { eprintln!("sort: invalid option -- '{}'", ch); return 1; }
                    }
                }
            }
            _ if arg.starts_with('-') => { eprintln!("sort: unrecognized option '{}'", arg); return 1; }
            _ => paths.push(args[i].clone()),
        }
        i += 1;
    }

    let opts = Opts { numeric, reverse, ignore_case, human_numeric, month_sort,
                      random_sort, stable, field_sep, keys };

    // Read all lines
    let mut lines: Vec<String> = Vec::new();
    let mut exit_code = 0;

    if paths.is_empty() {
        exit_code |= read_lines(&mut io::stdin().lock(), &mut lines);
    } else {
        for path in &paths {
            if path == "-" {
                exit_code |= read_lines(&mut io::stdin().lock(), &mut lines);
            } else {
                match File::open(path) {
                    Ok(f) => exit_code |= read_lines(&mut BufReader::new(f), &mut lines),
                    Err(e) => { eprintln!("sort: cannot read '{}': {}", path, e); exit_code = 1; }
                }
            }
        }
    }

    if check || check_quiet {
        for w in lines.windows(2) {
            if compare_lines(&w[1], &w[0], &opts) == std::cmp::Ordering::Less {
                if !check_quiet {
                    eprintln!("sort: disorder: {}", w[1]);
                }
                return 1;
            }
        }
        return 0;
    }

    if opts.random_sort {
        // Simple Fisher-Yates with a basic LCG
        let mut rng = simple_rng_seed();
        for i in (1..lines.len()).rev() {
            let j = simple_rng_next(&mut rng) % (i + 1);
            lines.swap(i, j);
        }
    } else if opts.stable {
        lines.sort_by(|a, b| compare_lines(a, b, &opts));
    } else {
        lines.sort_unstable_by(|a, b| compare_lines(a, b, &opts));
    }

    if unique {
        lines.dedup_by(|a, b| compare_lines(a, b, &opts) == std::cmp::Ordering::Equal);
    }

    let stdout;
    let file;
    let mut out: Box<dyn Write> = if let Some(ref path) = output {
        match std::fs::File::create(path) {
            Ok(f) => { file = f; Box::new(io::BufWriter::new(&file)) }
            Err(e) => { eprintln!("sort: cannot open '{}' for writing: {}", path, e); return 1; }
        }
    } else {
        stdout = io::stdout();
        Box::new(io::BufWriter::new(stdout.lock()))
    };

    for line in &lines {
        let _ = writeln!(out, "{}", line);
    }

    exit_code
}

struct Opts {
    numeric: bool,
    reverse: bool,
    ignore_case: bool,
    human_numeric: bool,
    month_sort: bool,
    random_sort: bool,
    stable: bool,
    field_sep: Option<u8>,
    keys: Vec<Key>,
}

struct Key {
    field_start: usize,
    char_start: usize,
    field_end: Option<usize>,
    char_end: Option<usize>,
    numeric: bool,
    reverse: bool,
    ignore_case: bool,
    human_numeric: bool,
    month_sort: bool,
}

fn compare_lines(a: &str, b: &str, opts: &Opts) -> std::cmp::Ordering {
    let ord = if !opts.keys.is_empty() {
        let mut ord = std::cmp::Ordering::Equal;
        for key in &opts.keys {
            let ka = extract_key(a, key, opts.field_sep);
            let kb = extract_key(b, key, opts.field_sep);
            ord = compare_strs(&ka, &kb, key.numeric || opts.numeric,
                key.ignore_case || opts.ignore_case,
                key.human_numeric || opts.human_numeric,
                key.month_sort || opts.month_sort);
            if key.reverse { ord = ord.reverse(); }
            if ord != std::cmp::Ordering::Equal { break; }
        }
        if ord == std::cmp::Ordering::Equal {
            compare_strs(a, b, opts.numeric, opts.ignore_case, opts.human_numeric, opts.month_sort)
        } else { ord }
    } else {
        compare_strs(a, b, opts.numeric, opts.ignore_case, opts.human_numeric, opts.month_sort)
    };
    if opts.reverse { ord.reverse() } else { ord }
}

fn compare_strs(a: &str, b: &str, numeric: bool, ignore_case: bool, human_numeric: bool, month_sort: bool) -> std::cmp::Ordering {
    if human_numeric {
        return parse_human(a).partial_cmp(&parse_human(b)).unwrap_or(std::cmp::Ordering::Equal);
    }
    if month_sort {
        return month_value(a).cmp(&month_value(b));
    }
    if numeric {
        let na = parse_numeric(a);
        let nb = parse_numeric(b);
        return na.partial_cmp(&nb).unwrap_or(std::cmp::Ordering::Equal);
    }
    if ignore_case {
        a.to_lowercase().cmp(&b.to_lowercase())
    } else {
        a.cmp(b)
    }
}

fn extract_key(line: &str, key: &Key, field_sep: Option<u8>) -> String {
    let sep = field_sep.map(|b| b as char).unwrap_or(' ');
    let fields: Vec<&str> = if field_sep.is_some() {
        line.split(sep).collect()
    } else {
        // whitespace: split on runs
        line.split_whitespace().collect()
    };

    let start_field = key.field_start.saturating_sub(1).min(fields.len());
    let start_char = key.char_start.saturating_sub(1);

    let end_field = key.field_end.map(|f| f.min(fields.len())).unwrap_or(fields.len());
    let end_char = key.char_end;

    if start_field >= fields.len() {
        return String::new();
    }

    let start_str = fields[start_field];
    let start_str = if start_char < start_str.len() {
        &start_str[start_char..]
    } else {
        ""
    };

    if key.field_end.is_none() {
        // From start to end of line
        let remaining: Vec<&str> = fields[start_field..].to_vec();
        let mut s = remaining.join(&sep.to_string());
        if start_char > 0 && !s.is_empty() {
            s = s[start_char.min(s.len())..].to_string();
        } else {
            s = start_str.to_string();
        }
        return s;
    }

    let end_field_idx = end_field.saturating_sub(1).min(fields.len().saturating_sub(1));
    if start_field == end_field_idx {
        let s = start_str;
        let s = if let Some(ec) = end_char {
            &s[..ec.min(s.len())]
        } else { s };
        return s.to_string();
    }

    let mut parts: Vec<&str> = fields[start_field..=end_field_idx].to_vec();
    if let Some(ec) = end_char {
        if let Some(last) = parts.last_mut() {
            *last = &last[..ec.min(last.len())];
        }
    }
    parts[0] = start_str;
    parts.join(&sep.to_string())
}

fn parse_key(s: &str) -> Option<Key> {

    // Parse: start[,end] where start/end = field[.char][flags]
    let (start_str, end_str) = if let Some(comma) = s.find(',') {
        (&s[..comma], Some(&s[comma+1..]))
    } else {
        (s, None)
    };

    let (sf, sc, n1, r1, ic1, hn1, ms1) = parse_key_pos(start_str)?;
    let (ef, ec, n2, r2, ic2, hn2, ms2) = if let Some(e) = end_str {
        let (f, c, n, r, ic, hn, ms) = parse_key_pos(e)?;
        (Some(f), Some(c), n, r, ic, hn, ms)
    } else {
        (None, None, false, false, false, false, false)
    };

    Some(Key {
        field_start: sf,
        char_start: sc,
        field_end: ef,
        char_end: if ec == Some(0) { None } else { ec },
        numeric: n1 || n2,
        reverse: r1 || r2,
        ignore_case: ic1 || ic2,
        human_numeric: hn1 || hn2,
        month_sort: ms1 || ms2,
    })
}

fn parse_key_pos(s: &str) -> Option<(usize, usize, bool, bool, bool, bool, bool)> {
    let mut num_str = String::new();
    let mut char_str = String::new();
    let mut flags = String::new();
    let mut in_char = false;
    let mut in_flags = false;

    for ch in s.chars() {
        if ch.is_ascii_digit() {
            if in_flags { return None; }
            if in_char { char_str.push(ch); } else { num_str.push(ch); }
        } else if ch == '.' {
            in_char = true;
        } else {
            in_flags = true;
            flags.push(ch);
        }
    }

    let field: usize = num_str.parse().ok().filter(|&n| n > 0)?;
    let char_off: usize = if char_str.is_empty() { 0 } else { char_str.parse().ok()? };

    let numeric = flags.contains('n');
    let reverse = flags.contains('r');
    let ignore_case = flags.contains('f');
    let human = flags.contains('h');
    let month = flags.contains('M');

    Some((field, char_off, numeric, reverse, ignore_case, human, month))
}

fn parse_numeric(s: &str) -> f64 {
    let s = s.trim_start();
    let s = s.trim_start_matches('+');
    s.parse::<f64>().unwrap_or(0.0)
}

fn parse_human(s: &str) -> f64 {
    let s = s.trim();
    if s.is_empty() { return 0.0; }
    let (num_part, suffix) = s.split_at(s.len() - 1);
    let multiplier = match suffix {
        "K" | "k" => 1_000f64,
        "M" => 1_000_000f64,
        "G" => 1_000_000_000f64,
        "T" => 1_000_000_000_000f64,
        "P" => 1_000_000_000_000_000f64,
        _ => return s.parse::<f64>().unwrap_or(0.0),
    };
    num_part.parse::<f64>().unwrap_or(0.0) * multiplier
}

fn month_value(s: &str) -> u8 {
    match s.trim().to_uppercase().get(..3).unwrap_or("") {
        "JAN" => 1, "FEB" => 2, "MAR" => 3, "APR" => 4,
        "MAY" => 5, "JUN" => 6, "JUL" => 7, "AUG" => 8,
        "SEP" => 9, "OCT" => 10, "NOV" => 11, "DEC" => 12,
        _ => 0,
    }
}

fn read_lines<R: BufRead>(reader: &mut R, lines: &mut Vec<String>) -> i32 {
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                let s = line.trim_end_matches('\n').trim_end_matches('\r').to_string();
                lines.push(s);
            }
            Err(_) => return 1,
        }
    }
    0
}
