use std::fs::File;
use std::io::{self, BufRead, BufReader};

pub fn run(args: &[String]) -> i32 {
    let mut count_lines = false;
    let mut count_words = false;
    let mut count_bytes = false;
    let mut count_chars = false;
    let mut count_max_line = false;
    let mut paths: Vec<String> = Vec::new();

    for arg in args {
        if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") {
            for ch in arg[1..].chars() {
                match ch {
                    'l' => count_lines = true,
                    'w' => count_words = true,
                    'c' => count_bytes = true,
                    'm' => count_chars = true,
                    'L' => count_max_line = true,
                    _ => {
                        eprintln!("wc: invalid option -- '{}'", ch);
                        return 1;
                    }
                }
            }
        } else {
            match arg.as_str() {
                "--lines" => count_lines = true,
                "--words" => count_words = true,
                "--bytes" => count_bytes = true,
                "--chars" => count_chars = true,
                "--max-line-length" => count_max_line = true,
                a if a.starts_with('-') => {
                    eprintln!("wc: unrecognized option '{}'", a);
                    return 1;
                }
                _ => paths.push(arg.clone()),
            }
        }
    }

    // Default: lines, words, bytes
    let default = !count_lines && !count_words && !count_bytes && !count_chars && !count_max_line;
    if default {
        count_lines = true;
        count_words = true;
        count_bytes = true;
    }

    let mut total = Counts::default();
    let mut exit_code = 0;
    let multiple = paths.len() > 1;
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    if paths.is_empty() {
        let counts = count_reader(&mut io::stdin().lock());
        print_counts(&counts, "", count_lines, count_words, count_bytes, count_chars, count_max_line, &mut out);
    } else {
        for path in &paths {
            let counts = if path == "-" {
                count_reader(&mut io::stdin().lock())
            } else {
                match File::open(path) {
                    Ok(f) => count_reader(&mut BufReader::new(f)),
                    Err(e) => {
                        eprintln!("wc: {}: {}", path, e);
                        exit_code = 1;
                        continue;
                    }
                }
            };
            total.lines += counts.lines;
            total.words += counts.words;
            total.bytes += counts.bytes;
            total.chars += counts.chars;
            total.max_line = total.max_line.max(counts.max_line);
            print_counts(&counts, path, count_lines, count_words, count_bytes, count_chars, count_max_line, &mut out);
        }
        if multiple {
            print_counts(&total, "total", count_lines, count_words, count_bytes, count_chars, count_max_line, &mut out);
        }
    }

    exit_code
}

#[derive(Default)]
struct Counts {
    lines: u64,
    words: u64,
    bytes: u64,
    chars: u64,
    max_line: u64,
}

fn count_reader<R: BufRead>(reader: &mut R) -> Counts {
    let mut counts = Counts::default();
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(n) => {
                counts.bytes += n as u64;
                counts.chars += line.chars().count() as u64;
                counts.lines += 1;
                counts.words += line.split_whitespace().count() as u64;
                let line_len = line.trim_end_matches('\n').trim_end_matches('\r').chars().count() as u64;
                counts.max_line = counts.max_line.max(line_len);
            }
            Err(_) => break,
        }
    }

    counts
}

#[allow(clippy::too_many_arguments)]
fn print_counts<W: io::Write>(
    counts: &Counts,
    label: &str,
    lines: bool,
    words: bool,
    bytes: bool,
    chars: bool,
    max_line: bool,
    out: &mut W,
) {
    let mut parts: Vec<String> = Vec::new();
    if lines     { parts.push(format!("{:>7}", counts.lines)); }
    if words     { parts.push(format!("{:>7}", counts.words)); }
    if bytes     { parts.push(format!("{:>7}", counts.bytes)); }
    if chars     { parts.push(format!("{:>7}", counts.chars)); }
    if max_line  { parts.push(format!("{:>7}", counts.max_line)); }

    if label.is_empty() {
        let _ = writeln!(out, "{}", parts.join(""));
    } else {
        let _ = writeln!(out, "{} {}", parts.join(""), label);
    }
}
