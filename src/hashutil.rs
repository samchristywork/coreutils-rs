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
