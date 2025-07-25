use std::io::{self, Write};

pub fn run(args: &[String]) -> i32 {
    let mut separator = "\n".to_string();
    let mut format: Option<String> = None;
    let mut equal_width = false;
    let mut vals: Vec<&str> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            "-w" | "--equal-width" => equal_width = true,
            "-s" | "--separator" => {
                i += 1;
                if i >= args.len() { eprintln!("seq: option requires an argument -- 's'"); return 1; }
                separator = unescape(&args[i]);
            }
            "-f" | "--format" => {
                i += 1;
                if i >= args.len() { eprintln!("seq: option requires an argument -- 'f'"); return 1; }
                format = Some(args[i].clone());
            }
            _ if arg.starts_with("--separator=") => separator = unescape(&arg["--separator=".len()..]),
            _ if arg.starts_with("--format=") => format = Some(arg["--format=".len()..].to_string()),
            _ if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") => {
                let mut chars = arg[1..].chars().peekable();
                while let Some(ch) = chars.next() {
                    match ch {
                        'w' => equal_width = true,
                        's' => {
                            let rest: String = chars.collect();
                            let val = if rest.is_empty() {
                                i += 1;
                                if i >= args.len() { eprintln!("seq: option requires an argument -- 's'"); return 1; }
                                unescape(&args[i])
                            } else { unescape(&rest) };
                            separator = val;
                            break;
                        }
                        'f' => {
                            let rest: String = chars.collect();
                            let val = if rest.is_empty() {
                                i += 1;
                                if i >= args.len() { eprintln!("seq: option requires an argument -- 'f'"); return 1; }
                                args[i].clone()
                            } else { rest };
                            format = Some(val);
                            break;
                        }
                        _ => { eprintln!("seq: invalid option -- '{}'", ch); return 1; }
                    }
                }
            }
            _ if arg.starts_with('-') => { eprintln!("seq: unrecognized option '{}'", arg); return 1; }
            _ => vals.push(arg),
        }
        i += 1;
    }

    let (first, incr, last) = match vals.len() {
        1 => (1.0f64, 1.0f64, match vals[0].parse::<f64>() {
            Ok(v) => v,
            Err(_) => { eprintln!("seq: invalid floating point argument '{}'", vals[0]); return 1; }
        }),
        2 => {
            let f = match vals[0].parse::<f64>() { Ok(v) => v, Err(_) => { eprintln!("seq: invalid floating point argument '{}'", vals[0]); return 1; } };
            let l = match vals[1].parse::<f64>() { Ok(v) => v, Err(_) => { eprintln!("seq: invalid floating point argument '{}'", vals[1]); return 1; } };
            (f, if f <= l { 1.0 } else { -1.0 }, l)
        }
        3 => {
            let f = match vals[0].parse::<f64>() { Ok(v) => v, Err(_) => { eprintln!("seq: invalid floating point argument '{}'", vals[0]); return 1; } };
            let s = match vals[1].parse::<f64>() { Ok(v) => v, Err(_) => { eprintln!("seq: invalid floating point argument '{}'", vals[1]); return 1; } };
            let l = match vals[2].parse::<f64>() { Ok(v) => v, Err(_) => { eprintln!("seq: invalid floating point argument '{}'", vals[2]); return 1; } };
            (f, s, l)
        }
        0 => { eprintln!("seq: missing operand"); return 1; }
        _ => { eprintln!("seq: extra operand '{}'", vals[3]); return 1; }
    };

    if incr == 0.0 {
        eprintln!("seq: increment must not be zero");
        return 1;
    }

    // Determine decimal places from inputs for default formatting
    let prec = if format.is_none() {
        max_decimals(&[vals[0], if vals.len() > 1 { vals[1] } else { vals[0] }, if vals.len() > 2 { vals[2] } else { vals[0] }])
    } else { 0 };

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    // Generate sequence
    let mut values: Vec<f64> = Vec::new();
    let mut n = 0i64;
    loop {
        let v = first + incr * n as f64;
        if incr > 0.0 && v > last + last.abs() * 1e-10 { break; }
        if incr < 0.0 && v < last - last.abs().max(first.abs()) * 1e-10 { break; }
        values.push(v);
        n += 1;
        if n > 10_000_000 { break; }
    }

    // Determine padding width for -w
    let width = if equal_width && format.is_none() {
        let fmt_str = if prec == 0 {
            format!("{:.0}", last)
        } else {
            format!("{:.prec$}", last, prec = prec)
        };
        fmt_str.len().max(format!("{:.0}", first).len())
    } else { 0 };

    for (idx, &v) in values.iter().enumerate() {
        if idx > 0 { let _ = out.write_all(separator.as_bytes()); }
        let s = if let Some(ref fmt) = format {
            apply_format(fmt, v)
        } else if prec == 0 {
            if equal_width {
                format!("{:0>width$.0}", v, width = width)
            } else {
                format!("{:.0}", v)
            }
        } else {
            format!("{:.prec$}", v, prec = prec)
        };
        let _ = out.write_all(s.as_bytes());
    }
    if !values.is_empty() {
        let _ = out.write_all(b"\n");
    }
    0
}

fn max_decimals(strs: &[&str]) -> usize {
    strs.iter().map(|s| {
        if let Some(pos) = s.find('.') { s.len() - pos - 1 } else { 0 }
    }).max().unwrap_or(0)
}

fn apply_format(fmt: &str, v: f64) -> String {
    // Support %f, %e, %g, %d and width/precision specifiers
    let mut out = String::new();
    let chars: Vec<char> = fmt.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] != '%' { out.push(chars[i]); i += 1; continue; }
        i += 1;
        // Collect flags/width/precision
        let start = i;
        while i < chars.len() && "0123456789.-+ ".contains(chars[i]) { i += 1; }
        let spec = &fmt[start..i];
        if i >= chars.len() { out.push('%'); break; }
        let conv = chars[i]; i += 1;
        let formatted = match conv {
            'f' | 'F' => {
                let prec = parse_prec(spec, 6);
                format!("{:.prec$}", v, prec = prec)
            }
            'e' => { let prec = parse_prec(spec, 6); format!("{:.prec$e}", v, prec = prec) }
            'g' | 'G' => format!("{}", v),
            'd' | 'i' => format!("{}", v as i64),
            '%' => { out.push('%'); continue; }
            _ => { out.push('%'); out.push(conv); continue; }
        };
        out.push_str(&formatted);
    }
    out
}

fn parse_prec(spec: &str, default: usize) -> usize {
    if let Some(pos) = spec.find('.') {
        spec[pos+1..].parse().unwrap_or(default)
    } else { default }
}

fn unescape(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('\\') => out.push('\\'),
                Some(c) => { out.push('\\'); out.push(c); }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}
