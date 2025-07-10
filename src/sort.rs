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
