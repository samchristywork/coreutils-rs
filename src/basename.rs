use std::io::{self, Write};

pub fn run(args: &[String]) -> i32 {
    let mut zero = false;
    let mut suffix: Option<String> = None;
    let mut multiple = false;
    let mut paths: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            "-z" | "--zero"     => zero = true,
            "-a" | "--multiple" => multiple = true,
            "-s" | "--suffix" => {
                i += 1;
                if i >= args.len() { eprintln!("basename: option requires an argument -- 's'"); return 1; }
                suffix = Some(args[i].clone());
                multiple = true;
            }
            _ if arg.starts_with("--suffix=") => {
                suffix = Some(arg["--suffix=".len()..].to_string());
                multiple = true;
            }
            _ if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") => {
                let mut chars = arg[1..].chars().peekable();
                while let Some(ch) = chars.next() {
                    match ch {
                        'z' => zero = true,
                        'a' => multiple = true,
                        's' => {
                            let rest: String = chars.collect();
                            let val = if rest.is_empty() {
                                i += 1;
                                if i >= args.len() { eprintln!("basename: option requires an argument -- 's'"); return 1; }
                                args[i].clone()
                            } else { rest };
                            suffix = Some(val);
                            multiple = true;
                            break;
                        }
                        _ => { eprintln!("basename: invalid option -- '{}'", ch); return 1; }
                    }
                }
            }
            _ if arg.starts_with('-') => { eprintln!("basename: unrecognized option '{}'", arg); return 1; }
            _ => paths.push(arg.to_string()),
        }
        i += 1;
    }

    if paths.is_empty() {
        eprintln!("basename: missing operand");
        return 1;
    }

    // Without -a or -s: basename NAME [SUFFIX]
    if !multiple {
        if paths.len() > 2 {
            eprintln!("basename: extra operand '{}'", paths[2]);
            return 1;
        }
        let suf = if paths.len() == 2 { Some(paths[1].as_str()) } else { None };
        let result = compute(&paths[0], suf);
        if zero { print!("{}\0", result); } else { println!("{}", result); }
        return 0;
    }

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    for path in &paths {
        let result = compute(path, suffix.as_deref());
        if zero {
            let _ = write!(out, "{}\0", result);
        } else {
            let _ = writeln!(out, "{}", result);
        }
    }
    0
}

fn compute(path: &str, suffix: Option<&str>) -> String {
    // Strip trailing slashes (but keep root "/")
    let trimmed = path.trim_end_matches('/');
    let trimmed = if trimmed.is_empty() { "/" } else { trimmed };

    // Take the last component
    let base = match trimmed.rfind('/') {
        Some(pos) => &trimmed[pos + 1..],
        None      => trimmed,
    };
    let base = if base.is_empty() { "/" } else { base };

    // Strip suffix if it matches (and would not empty the result)
    if let Some(suf) = suffix {
        if base != suf && base.ends_with(suf) {
            return base[..base.len() - suf.len()].to_string();
        }
    }
    base.to_string()
}
