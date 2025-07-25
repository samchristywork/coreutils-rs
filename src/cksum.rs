use std::io::{self, Read, Write};
use std::fs::File;

pub fn run(args: &[String]) -> i32 {
    let mut paths: Vec<String> = Vec::new();

    for arg in args {
        if arg.starts_with('-') {
            eprintln!("cksum: unrecognized option '{}'", arg);
            return 1;
        }
        paths.push(arg.clone());
    }

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let mut exit_code = 0;

    if paths.is_empty() {
        let (crc, nbytes) = compute(&mut io::stdin());
        let _ = writeln!(out, "{} {}", crc, nbytes);
    } else {
        for path in &paths {
            let result = if path == "-" {
                compute(&mut io::stdin())
            } else {
                match File::open(path) {
                    Ok(mut f) => compute(&mut f),
                    Err(e) => { eprintln!("cksum: {}: {}", path, e); exit_code = 1; continue; }
                }
            };
            let (crc, nbytes) = result;
            let _ = writeln!(out, "{} {} {}", crc, nbytes, path);
        }
    }
    exit_code
}
