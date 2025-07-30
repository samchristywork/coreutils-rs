use std::io::{self, Write};
use crate::echo::process_escapes;

pub fn run(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("printf: missing operand");
        return 1;
    }

    let fmt = &args[0];
    let mut operands = &args[1..];
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    // Repeat format until all operands consumed; at least one pass
    let mut first = true;
    loop {
        let (output, used) = format_once(fmt, operands);
        let _ = out.write_all(output.as_bytes());
        if used == 0 || operands.is_empty() {
            if first && operands.is_empty() { break; }
            if !first { break; }
        }
        operands = &operands[used.min(operands.len())..];
        first = false;
        if operands.is_empty() { break; }
    }
    0
}

// Format one pass of fmt consuming as many operands as needed.
// Returns (output_string, number_of_operands_consumed).
fn format_once(fmt: &str, operands: &[String]) -> (String, usize) {
    let mut out = String::new();
    let mut op_idx = 0;
    let mut chars = fmt.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.peek().copied() {
                Some('n')  => { out.push('\n'); chars.next(); }
                Some('t')  => { out.push('\t'); chars.next(); }
                Some('r')  => { out.push('\r'); chars.next(); }
                Some('a')  => { out.push('\x07'); chars.next(); }
                Some('b')  => { out.push('\x08'); chars.next(); }
                Some('f')  => { out.push('\x0C'); chars.next(); }
                Some('v')  => { out.push('\x0B'); chars.next(); }
                Some('e')  => { out.push('\x1B'); chars.next(); }
                Some('\\') => { out.push('\\'); chars.next(); }
                Some('0') | Some('1') | Some('2') | Some('3') |
                Some('4') | Some('5') | Some('6') | Some('7') => {
                    let mut oct = String::new();
                    for _ in 0..3 {
                        match chars.peek() {
                            Some(&d) if d >= '0' && d <= '7' => { oct.push(d); chars.next(); }
                            _ => break,
                        }
                    }
                    let val = u8::from_str_radix(&oct, 8).unwrap_or(0);
                    out.push(val as char);
                }
                _ => { out.push('\\'); }
            }
            continue;
        }

        if c != '%' { out.push(c); continue; }

        // Collect format spec: %[flags][width][.prec]conv
        let mut flags = String::new();
        while let Some(&ch) = chars.peek() {
            if "-+ #0".contains(ch) { flags.push(ch); chars.next(); } else { break; }
        }
        let mut width_str = String::new();
        while let Some(&ch) = chars.peek() {
            if ch.is_ascii_digit() { width_str.push(ch); chars.next(); } else { break; }
        }
        let mut prec_str: Option<String> = None;
        if chars.peek() == Some(&'.') {
            chars.next();
            let mut p = String::new();
            while let Some(&ch) = chars.peek() {
                if ch.is_ascii_digit() { p.push(ch); chars.next(); } else { break; }
            }
            prec_str = Some(p);
        }

        let conv = match chars.next() {
            Some(c) => c,
            None => break,
        };

        if conv == '%' { out.push('%'); continue; }

        let arg = operands.get(op_idx).map(|s| s.as_str()).unwrap_or("");
        if op_idx < operands.len() { op_idx += 1; }

        let width: usize = width_str.parse().unwrap_or(0);
        let left_align = flags.contains('-');

        match conv {
            's' => {
                let s = process_escapes(arg); // %b behaviour; %s doesn't escape but close enough
                let s = if conv == 's' { arg.to_string() } else { s };
                let s = if let Some(ref p) = prec_str {
                    let n: usize = p.parse().unwrap_or(s.len());
                    s.chars().take(n).collect()
                } else { s };
                out.push_str(&pad(&s, width, left_align, ' '));
            }
            'b' => {
                let s = process_escapes(arg);
                out.push_str(&pad(&s, width, left_align, ' '));
            }
            'c' => {
                let ch = arg.chars().next().unwrap_or('\0');
                out.push(ch);
            }
            'd' | 'i' => {
                let n: i64 = parse_int_arg(arg);
                let s = format!("{}", n);
                let s = apply_numeric_flags(&s, &flags, width, prec_str.as_deref(), false);
                out.push_str(&s);
            }
            'u' => {
                let n: u64 = parse_uint_arg(arg);
                let s = format!("{}", n);
                let s = apply_numeric_flags(&s, &flags, width, prec_str.as_deref(), false);
                out.push_str(&s);
            }
            'o' => {
                let n: u64 = parse_uint_arg(arg);
                let s = if flags.contains('#') { format!("0{:o}", n) } else { format!("{:o}", n) };
                let s = apply_numeric_flags(&s, &flags, width, prec_str.as_deref(), false);
                out.push_str(&s);
            }
            'x' => {
                let n: u64 = parse_uint_arg(arg);
                let s = if flags.contains('#') { format!("0x{:x}", n) } else { format!("{:x}", n) };
                let s = apply_numeric_flags(&s, &flags, width, prec_str.as_deref(), false);
                out.push_str(&s);
            }
            'X' => {
                let n: u64 = parse_uint_arg(arg);
                let s = if flags.contains('#') { format!("0X{:X}", n) } else { format!("{:X}", n) };
                let s = apply_numeric_flags(&s, &flags, width, prec_str.as_deref(), false);
                out.push_str(&s);
            }
            'f' | 'F' => {
                let n: f64 = arg.parse().unwrap_or(0.0);
                let prec: usize = prec_str.as_deref().and_then(|p| p.parse().ok()).unwrap_or(6);
                let s = format!("{:.prec$}", n, prec = prec);
                let s = apply_numeric_flags(&s, &flags, width, None, true);
                out.push_str(&s);
            }
            'e' | 'E' => {
                let n: f64 = arg.parse().unwrap_or(0.0);
                let prec: usize = prec_str.as_deref().and_then(|p| p.parse().ok()).unwrap_or(6);
                let s = format!("{:.prec$e}", n, prec = prec);
                let s = if conv == 'E' { s.to_uppercase() } else { s };
                let s = apply_numeric_flags(&s, &flags, width, None, true);
                out.push_str(&s);
            }
            'g' | 'G' => {
                let n: f64 = arg.parse().unwrap_or(0.0);
                let s = format!("{}", n);
                let s = if conv == 'G' { s.to_uppercase() } else { s };
                out.push_str(&pad(&s, width, left_align, ' '));
            }
            _ => { out.push('%'); out.push(conv); }
        }
    }

    (out, op_idx)
}
