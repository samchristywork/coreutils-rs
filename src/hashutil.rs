use std::io::{self, Read, BufRead, Write};
use std::fs::File;

// Common runner for md5sum / sha*sum style commands.
// compute_fn streams from a reader and returns a lowercase hex digest.
pub fn run(args: &[String], compute_fn: fn(&mut dyn Read) -> io::Result<String>) -> i32 {
    let mut check = false;
    let mut binary = false;
    let mut quiet = false;
    let mut status_only = false;
    let mut warn = false;
    let mut paths: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            "-c" | "--check"        => check = true,
            "-b" | "--binary"       => binary = true,
            "-t" | "--text"         => binary = false,
            "-q" | "--quiet"        => quiet = true,
            "--status"              => status_only = true,
            "-w" | "--warn"         => warn = true,
            _ if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") => {
                for ch in arg[1..].chars() {
                    match ch {
                        'c' => check = true,
                        'b' => binary = true,
                        't' => binary = false,
                        'q' => quiet = true,
                        'w' => warn = true,
                        _ => { eprintln!("invalid option -- '{}'", ch); return 1; }
                    }
                }
            }
            _ if arg.starts_with('-') => { eprintln!("unrecognized option '{}'", arg); return 1; }
            _ => paths.push(arg.to_string()),
        }
        i += 1;
    }

    if check {
        run_check(&paths, compute_fn, quiet, status_only, warn)
    } else {
        run_hash(&paths, compute_fn, binary)
    }
}

fn run_hash(paths: &[String], compute_fn: fn(&mut dyn Read) -> io::Result<String>, binary: bool) -> i32 {
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let mut exit_code = 0;

    let mode_char = if binary { '*' } else { ' ' };

    if paths.is_empty() {
        match compute_fn(&mut io::stdin()) {
            Ok(digest) => { let _ = writeln!(out, "{}  -", digest); }
            Err(e) => { eprintln!("-: {}", e); exit_code = 1; }
        }
    } else {
        for path in paths {
            let result = if path == "-" {
                compute_fn(&mut io::stdin())
            } else {
                File::open(path).and_then(|mut f| compute_fn(&mut f))
            };
            match result {
                Ok(digest) => { let _ = writeln!(out, "{} {}{}", digest, mode_char, path); }
                Err(e) => { eprintln!("{}: {}", path, e); exit_code = 1; }
            }
        }
    }
    exit_code
}
