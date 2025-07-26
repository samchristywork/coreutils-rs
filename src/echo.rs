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
