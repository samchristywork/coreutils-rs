use std::collections::HashSet;
use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::MetadataExt;
use std::path::Path;

pub fn run(args: &[String]) -> i32 {
    let mut all = false;
    let mut summarize = false;
    let mut human = false;
    let mut block_size: u64 = 1024;
    let mut max_depth: Option<usize> = None;
    let mut one_fs = false;
    let mut paths: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            "-a" | "--all"             => all = true,
            "-s" | "--summarize"       => summarize = true,
            "-h" | "--human-readable"  => human = true,
            "-k"                       => block_size = 1024,
            "-m"                       => block_size = 1024 * 1024,
            "-x" | "--one-file-system" => one_fs = true,
            "--max-depth" => {
                i += 1;
                if i >= args.len() { eprintln!("du: option requires an argument -- 'max-depth'"); return 1; }
                match args[i].parse() {
                    Ok(n) => max_depth = Some(n),
                    Err(_) => { eprintln!("du: invalid max depth '{}'", args[i]); return 1; }
                }
            }
            _ if arg.starts_with("--max-depth=") => {
                match arg["--max-depth=".len()..].parse() {
                    Ok(n) => max_depth = Some(n),
                    Err(_) => { eprintln!("du: invalid max depth"); return 1; }
                }
            }
            _ if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") => {
                let mut chars = arg[1..].chars().peekable();
                while let Some(ch) = chars.next() {
                    match ch {
                        'a' => all = true,
                        's' => summarize = true,
                        'h' => human = true,
                        'k' => block_size = 1024,
                        'm' => block_size = 1024 * 1024,
                        'x' => one_fs = true,
                        'd' => {
                            let rest: String = chars.collect();
                            let val = if rest.is_empty() {
                                i += 1;
                                if i >= args.len() { eprintln!("du: option requires an argument -- 'd'"); return 1; }
                                args[i].clone()
                            } else { rest };
                            match val.parse() {
                                Ok(n) => max_depth = Some(n),
                                Err(_) => { eprintln!("du: invalid max depth '{}'", val); return 1; }
                            }
                            break;
                        }
                        _ => { eprintln!("du: invalid option -- '{}'", ch); return 1; }
                    }
                }
            }
            _ if arg.starts_with('-') => { eprintln!("du: unrecognized option '{}'", arg); return 1; }
            _ => paths.push(args[i].clone()),
        }
        i += 1;
    }

    if paths.is_empty() { paths.push(".".to_string()); }

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let mut seen_inodes: HashSet<(u64, u64)> = HashSet::new();
    let mut exit_code = 0;

    for path in &paths {
        let p = Path::new(path);
        match du_path(p, 0, all, summarize, human, block_size, max_depth, one_fs, &mut seen_inodes, &mut out) {
            Ok(_) => {}
            Err(_) => exit_code = 1,
        }
    }
    exit_code
}
