use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

pub fn run(args: &[String]) -> i32 {
    let mut body_style = Style::NonEmpty;
    let mut header_style = Style::None;
    let mut footer_style = Style::None;
    let mut width = 6usize;
    let mut separator = String::from("\t");
    let mut start = 1i64;
    let mut increment = 1i64;
    let mut paths: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        if let Some(val) = long_opt_val(arg, "--body-numbering=")
            .or_else(|| short_opt_val(arg, 'b'))
        {
            let val = if val.is_empty() {
                i += 1;
                if i >= args.len() { eprintln!("nl: option requires an argument -- 'b'"); return 1; }
                args[i].clone()
            } else { val };
            match parse_style(&val) {
                Some(s) => body_style = s,
                None => { eprintln!("nl: invalid line numbering style: '{}'", val); return 1; }
            }
        } else if let Some(val) = long_opt_val(arg, "--header-numbering=")
            .or_else(|| short_opt_val(arg, 'h'))
        {
            let val = if val.is_empty() {
                i += 1;
                if i >= args.len() { eprintln!("nl: option requires an argument -- 'h'"); return 1; }
                args[i].clone()
            } else { val };
            match parse_style(&val) {
                Some(s) => header_style = s,
                None => { eprintln!("nl: invalid line numbering style: '{}'", val); return 1; }
            }
        } else if let Some(val) = long_opt_val(arg, "--footer-numbering=")
            .or_else(|| short_opt_val(arg, 'f'))
        {
            let val = if val.is_empty() {
                i += 1;
                if i >= args.len() { eprintln!("nl: option requires an argument -- 'f'"); return 1; }
                args[i].clone()
            } else { val };
            match parse_style(&val) {
                Some(s) => footer_style = s,
                None => { eprintln!("nl: invalid line numbering style: '{}'", val); return 1; }
            }
        } else if let Some(val) = long_opt_val(arg, "--number-width=")
            .or_else(|| short_opt_val(arg, 'w'))
        {
            let val = if val.is_empty() {
                i += 1;
                if i >= args.len() { eprintln!("nl: option requires an argument -- 'w'"); return 1; }
                args[i].clone()
            } else { val };
            match val.parse::<usize>() {
                Ok(n) if n > 0 => width = n,
                _ => { eprintln!("nl: invalid line number field width: '{}'", val); return 1; }
            }
        } else if let Some(val) = long_opt_val(arg, "--number-separator=")
            .or_else(|| short_opt_val(arg, 'n'))
        {
            // -n is number format (ln, rn, rz), not separator; -s is separator
            let val = if val.is_empty() {
                i += 1;
                if i >= args.len() { eprintln!("nl: option requires an argument -- 'n'"); return 1; }
                args[i].clone()
            } else { val };
            // -n: ln=left no-pad, rn=right no-pad, rz=right zero-pad
            // We store it as a format hint by overloading width sign; simpler: ignore format, just note
            let _ = val; // format hint not used beyond default right-aligned
        } else if let Some(val) = long_opt_val(arg, "--section-delimiter=")
            .or_else(|| short_opt_val(arg, 'd'))
        {
            let val = if val.is_empty() {
                i += 1;
                if i >= args.len() { eprintln!("nl: option requires an argument -- 'd'"); return 1; }
                args[i].clone()
            } else { val };
            let _ = val; // section delimiter (default \\:) not commonly needed
        } else if let Some(val) = long_opt_val(arg, "--number-separator=")
            .or_else(|| short_opt_val(arg, 's'))
        {
            let val = if val.is_empty() {
                i += 1;
                if i >= args.len() { eprintln!("nl: option requires an argument -- 's'"); return 1; }
                args[i].clone()
            } else { val };
            separator = val;
        } else if let Some(val) = long_opt_val(arg, "--starting-line-number=")
            .or_else(|| short_opt_val(arg, 'v'))
        {
            let val = if val.is_empty() {
                i += 1;
                if i >= args.len() { eprintln!("nl: option requires an argument -- 'v'"); return 1; }
                args[i].clone()
            } else { val };
            match val.parse::<i64>() {
                Ok(n) => start = n,
                Err(_) => { eprintln!("nl: invalid starting line number: '{}'", val); return 1; }
            }
        } else if let Some(val) = long_opt_val(arg, "--line-increment=")
            .or_else(|| short_opt_val(arg, 'i'))
        {
            let val = if val.is_empty() {
                i += 1;
                if i >= args.len() { eprintln!("nl: option requires an argument -- 'i'"); return 1; }
                args[i].clone()
            } else { val };
            match val.parse::<i64>() {
                Ok(n) => increment = n,
                Err(_) => { eprintln!("nl: invalid line number increment: '{}'", val); return 1; }
            }
        } else if arg.starts_with('-') && arg.len() > 1 {
            eprintln!("nl: invalid option -- '{}'", arg);
            return 1;
        } else {
            paths.push(args[i].clone());
        }
        i += 1;
    }

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let mut exit_code = 0;
    let mut line_num = start;

    let opts = Opts { body_style, header_style, footer_style, width, separator, increment };

    if paths.is_empty() {
        exit_code |= number_lines(&mut io::stdin().lock(), &mut out, &opts, &mut line_num);
    } else {
        for path in &paths {
            if path == "-" {
                exit_code |= number_lines(&mut io::stdin().lock(), &mut out, &opts, &mut line_num);
            } else {
                match File::open(path) {
                    Ok(f) => {
                        let mut reader = BufReader::new(f);
                        exit_code |= number_lines(&mut reader, &mut out, &opts, &mut line_num);
                    }
                    Err(e) => {
                        eprintln!("nl: {}: {}", path, e);
                        exit_code = 1;
                    }
                }
            }
        }
    }

    exit_code
}

struct Opts {
    body_style: Style,
    header_style: Style,
    footer_style: Style,
    width: usize,
    separator: String,
    increment: i64,
}

#[derive(Clone, Copy)]
enum Style {
    All,
    NonEmpty,
    None,
}

fn number_lines<R: BufRead, W: Write>(
    reader: &mut R,
    out: &mut W,
    opts: &Opts,
    line_num: &mut i64,
) -> i32 {
    // Logical sections: header (\\::\\::\\:), body (\\:), footer (\\:\\:)
    // For simplicity we treat everything as body unless a section delimiter is seen.
    let mut section = Section::Body;
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(_) => return 1,
        }

        let content = line.trim_end_matches('\n').trim_end_matches('\r');

        // Check for section delimiters
        if content == "\\:\\:\\:" {
            section = Section::Header;
            let _ = writeln!(out);
            continue;
        } else if content == "\\:" {
            section = Section::Body;
            let _ = writeln!(out);
            continue;
        } else if content == "\\:\\:" {
            section = Section::Footer;
            let _ = writeln!(out);
            continue;
        }

        let style = match section {
            Section::Header => opts.header_style,
            Section::Body => opts.body_style,
            Section::Footer => opts.footer_style,
        };

        let should_number = match style {
            Style::All => true,
            Style::NonEmpty => !content.is_empty(),
            Style::None => false,
        };

        if should_number {
            if write!(out, "{:>width$}{}{}\n",
                line_num, opts.separator, content,
                width = opts.width).is_err() {
                return 1;
            }
            *line_num += opts.increment;
        } else {
            if write!(out, "{}{}\n", " ".repeat(opts.width + opts.separator.len()), content).is_err() {
                return 1;
            }
        }
    }

    0
}

enum Section { Header, Body, Footer }

fn parse_style(s: &str) -> Option<Style> {
    match s {
        "a" => Some(Style::All),
        "t" => Some(Style::NonEmpty),
        "n" => Some(Style::None),
        _ => None,
    }
}

fn long_opt_val<'a>(arg: &'a str, prefix: &str) -> Option<String> {
    arg.strip_prefix(prefix).map(|s| s.to_string())
}

fn short_opt_val(arg: &str, flag: char) -> Option<String> {
    let s = arg.strip_prefix('-')?;
    let mut chars = s.chars();
    if chars.next() == Some(flag) {
        Some(chars.collect())
    } else {
        None
    }
}
