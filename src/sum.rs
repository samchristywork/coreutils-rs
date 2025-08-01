use std::io::{self, Read, Write};
use std::fs::File;

pub fn run(args: &[String]) -> i32 {
    let mut sysv = false;
    let mut paths: Vec<String> = Vec::new();

    for arg in args {
        match arg.as_str() {
            "-s" | "--sysv" => sysv = true,
            "-r"            => sysv = false,
            _ if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") => {
                for ch in arg[1..].chars() {
                    match ch {
                        's' => sysv = true,
                        'r' => sysv = false,
                        _ => { eprintln!("sum: invalid option -- '{}'", ch); return 1; }
                    }
                }
            }
            _ if arg.starts_with('-') => { eprintln!("sum: unrecognized option '{}'", arg); return 1; }
            _ => paths.push(arg.clone()),
        }
    }

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let mut exit_code = 0;

    if paths.is_empty() {
        let (cksum, blocks) = compute(&mut io::stdin(), sysv);
        let _ = writeln!(out, "{:05} {:5}", cksum, blocks);
    } else {
        for path in &paths {
            let result = if path == "-" {
                compute(&mut io::stdin(), sysv)
            } else {
                match File::open(path) {
                    Ok(mut f) => compute(&mut f, sysv),
                    Err(e) => { eprintln!("sum: {}: {}", path, e); exit_code = 1; continue; }
                }
            };
            let (cksum, blocks) = result;
            if paths.len() == 1 {
                let _ = writeln!(out, "{:05} {:5}", cksum, blocks);
            } else {
                let _ = writeln!(out, "{:05} {:5} {}", cksum, blocks, path);
            }
        }
    }
    exit_code
}
