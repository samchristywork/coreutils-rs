use std::fs;
use std::io::{self, Read, Write};
use std::os::unix::io::AsRawFd;

use crate::term::{is_tty, read_fd, Key, Term};

pub fn run(args: &[String]) -> i32 {
    let mut paths: Vec<String> = Vec::new();

    for arg in args {
        if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") {
            if let Some(ch) = arg[1..].chars().next() {
                eprintln!("more: invalid option -- '{}'", ch);
                return 1;
            }
        } else if arg.starts_with('-') && arg != "-" {
            eprintln!("more: unrecognized option '{}'", arg);
            return 1;
        } else {
            paths.push(arg.clone());
        }
    }

    let content = if paths.is_empty() {
        let mut buf = String::new();
        if io::stdin().read_to_string(&mut buf).is_err() {
            eprintln!("more: error reading stdin");
            return 1;
        }
        buf
    } else {
        let mut combined = String::new();
        for path in &paths {
            if path == "-" {
                if io::stdin().read_to_string(&mut combined).is_err() {
                    eprintln!("more: error reading stdin");
                    return 1;
                }
            } else {
                match fs::read_to_string(path) {
                    Ok(s) => combined.push_str(&s),
                    Err(e) => {
                        eprintln!("more: {}: {}", path, e);
                        return 1;
                    }
                }
            }
        }
        combined
    };

    let lines: Vec<&str> = content.split('\n').collect();
    let lines = if lines.last() == Some(&"") {
        &lines[..lines.len() - 1]
    } else {
        &lines[..]
    };

    if !is_tty() {
        let stdout = io::stdout();
        let mut out = io::BufWriter::new(stdout.lock());
        for line in lines {
            let _ = writeln!(out, "{}", line);
        }
        return 0;
    }

    let mut term = match Term::open() {
        Some(t) => t,
        None => {
            eprintln!("more: could not open terminal");
            return 1;
        }
    };

    page(&mut term, lines)
}

fn page(term: &mut Term, lines: &[&str]) -> i32 {
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    let mut pos = 0;

    loop {
        let (rows, cols) = term.size();
        let page_size = rows.saturating_sub(1);

        // Print a page of content
        let end = (pos + page_size).min(lines.len());
        for line in &lines[pos..end] {
            let truncated = truncate_str(line, cols);
            let _ = write!(out, "{}\r\n", truncated);
        }
        let _ = out.flush();

        pos = end;

        if pos >= lines.len() {
            break;
        }

        // Draw prompt
        let pct = pos * 100 / lines.len().max(1);
        let prompt = format!("\x1b[7m--More-- ({}%)\x1b[0m", pct);
        {
            let mut tty = &term.tty;
            write!(tty, "{}", prompt).ok();
            tty.flush().ok();
        }

        // Wait for keypress
        let mut buf = [0u8; 8];
        let n = read_fd(term.tty.as_raw_fd(), &mut buf).unwrap_or(0);
        term.last_byte = if n > 0 { buf[0] } else { 0 };

        // Erase prompt line
        {
            let mut tty = &term.tty;
            write!(tty, "\r\x1b[2K").ok();
            tty.flush().ok();
        }

        let key = if n > 0 {
            match &buf[..n] {
                b"q" | b"Q" | b"\x1b" => Key::Quit,
                b" " | b"\x1b[6~" => Key::Space,
                b"\r" | b"\n" | b"\x1b[B" | b"j" => Key::Enter,
                b"b" | b"\x1b[5~" => Key::B,
                _ => Key::Unknown,
            }
        } else {
            Key::Unknown
        };

        match key {
            Key::Quit => break,
            Key::Space => {} // already advanced a full page
            Key::Enter | Key::J => {
                // One line at a time: step back (page_size - 1) so next iteration shows +1
                pos = pos.saturating_sub(page_size.saturating_sub(1));
            }
            Key::B => {
                pos = pos.saturating_sub(page_size * 2);
            }
            _ => {}
        }
    }

    0
}

fn truncate_str(s: &str, max_cols: usize) -> &str {
    if max_cols == 0 {
        return "";
    }
    let mut end = s.len();
    for (width, (i, _)) in s.char_indices().enumerate() {
        if width >= max_cols {
            end = i;
            break;
        }
    }
    &s[..end]
}
