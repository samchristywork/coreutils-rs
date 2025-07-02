use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};

pub fn run(args: &[String]) -> i32 {
    let mut number_lines = false;
    let mut number_nonblank = false;
    let mut show_ends = false;
    let mut show_tabs = false;
    let mut squeeze_blank = false;
    let mut paths: Vec<String> = Vec::new();

    for arg in args {
        if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") {
            for ch in arg[1..].chars() {
                match ch {
                    'n' => number_lines = true,
                    'b' => number_nonblank = true,
                    'E' => show_ends = true,
                    'T' => show_tabs = true,
                    's' => squeeze_blank = true,
                    'A' => { show_ends = true; show_tabs = true; }
                    'e' => show_ends = true,
                    't' => show_tabs = true,
                    'v' => {} // show non-printing - skip for now (pass through)
                    _ => {
                        eprintln!("cat: invalid option -- '{}'", ch);
                        return 1;
                    }
                }
            }
        } else {
            match arg.as_str() {
                "--number" => number_lines = true,
                "--number-nonblank" => number_nonblank = true,
                "--show-ends" => show_ends = true,
                "--show-tabs" => show_tabs = true,
                "--squeeze-blank" => squeeze_blank = true,
                "--show-all" => { show_ends = true; show_tabs = true; }
                a if a.starts_with('-') => {
                    eprintln!("cat: unrecognized option '{}'", a);
                    return 1;
                }
                _ => paths.push(arg.clone()),
            }
        }
    }

    // -b overrides -n
    if number_nonblank {
        number_lines = false;
    }

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let mut exit_code = 0;

    if paths.is_empty() {
        exit_code |= cat_reader(
            &mut io::stdin().lock(),
            &mut out,
            number_lines,
            number_nonblank,
            show_ends,
            show_tabs,
            squeeze_blank,
            &mut 1,
        );
    } else {
        let mut line_num = 1usize;
        for path in &paths {
            if path == "-" {
                exit_code |= cat_reader(
                    &mut io::stdin().lock(),
                    &mut out,
                    number_lines,
                    number_nonblank,
                    show_ends,
                    show_tabs,
                    squeeze_blank,
                    &mut line_num,
                );
            } else {
                match File::open(path) {
                    Ok(f) => {
                        let mut reader = BufReader::new(f);
                        exit_code |= cat_reader(
                            &mut reader,
                            &mut out,
                            number_lines,
                            number_nonblank,
                            show_ends,
                            show_tabs,
                            squeeze_blank,
                            &mut line_num,
                        );
                    }
                    Err(e) => {
                        eprintln!("cat: {}: {}", path, e);
                        exit_code = 1;
                    }
                }
            }
        }
    }

    exit_code
}
