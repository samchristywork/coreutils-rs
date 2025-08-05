use std::io::{self, Write};

pub fn run(args: &[String]) -> i32 {
    let mut newline = true;
    let mut escape = false;
    let mut i = 0;

    // Parse leading flags
    while i < args.len() {
        let arg = args[i].as_str();
        if !arg.starts_with('-') || arg.len() < 2 { break; }
        let flags = &arg[1..];
        if flags.chars().all(|c| c == 'n' || c == 'e' || c == 'E') {
            for ch in flags.chars() {
                match ch {
                    'n' => newline = false,
                    'e' => escape = true,
                    'E' => escape = false,
                    _ => unreachable!(),
                }
            }
            i += 1;
        } else {
            break;
        }
    }

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    let mut first = true;
    while i < args.len() {
        if !first { let _ = out.write_all(b" "); }
        first = false;
        let s = &args[i];
        if escape {
            let _ = out.write_all(process_escapes(s).as_bytes());
        } else {
            let _ = out.write_all(s.as_bytes());
        }
        i += 1;
    }
    if newline { let _ = out.write_all(b"\n"); }
    0
}

pub fn process_escapes(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\\' { out.push(c); continue; }
        match chars.next() {
            Some('\\') => out.push('\\'),
            Some('n')  => out.push('\n'),
            Some('t')  => out.push('\t'),
            Some('r')  => out.push('\r'),
            Some('a')  => out.push('\x07'),
            Some('b')  => out.push('\x08'),
            Some('f')  => out.push('\x0C'),
            Some('v')  => out.push('\x0B'),
            Some('e')  => out.push('\x1B'),
            Some('0') => {
                // Octal: up to 3 digits
                let mut oct = String::new();
                for _ in 0..3 {
                    match chars.peek() {
                        Some(&d) if ('0'..='7').contains(&d) => { oct.push(d); chars.next(); }
                        _ => break,
                    }
                }
                let val = u8::from_str_radix(&oct, 8).unwrap_or(0);
                out.push(val as char);
            }
            Some('x') => {
                // Hex: up to 2 digits
                let mut hex = String::new();
                for _ in 0..2 {
                    match chars.peek() {
                        Some(&d) if d.is_ascii_hexdigit() => { hex.push(d); chars.next(); }
                        _ => break,
                    }
                }
                if !hex.is_empty() {
                    let val = u8::from_str_radix(&hex, 16).unwrap_or(0);
                    out.push(val as char);
                } else {
                    out.push('\\'); out.push('x');
                }
            }
            Some('c') => break, // suppress further output
            Some(c)  => { out.push('\\'); out.push(c); }
            None     => out.push('\\'),
        }
    }
    out
}
