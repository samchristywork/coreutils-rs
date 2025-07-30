use std::fs;
use std::path::Path;

pub fn run(args: &[String]) -> i32 {
    let mut canonicalize = false;
    let mut canonicalize_missing = false;
    let mut no_newline = false;
    let mut quiet = false;
    let mut paths: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            "-f" | "--canonicalize"         => canonicalize = true,
            "-m" | "--canonicalize-missing" => canonicalize_missing = true,
            "-n" | "--no-newline"           => no_newline = true,
            "-q" | "--quiet" | "--silent"   => quiet = true,
            _ if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") => {
                for ch in arg[1..].chars() {
                    match ch {
                        'f' => canonicalize = true,
                        'm' => canonicalize_missing = true,
                        'n' => no_newline = true,
                        'q' => quiet = true,
                        _ => { eprintln!("readlink: invalid option -- '{}'", ch); return 1; }
                    }
                }
            }
            _ if arg.starts_with('-') => { eprintln!("readlink: unrecognized option '{}'", arg); return 1; }
            _ => paths.push(arg.to_string()),
        }
        i += 1;
    }

    if paths.is_empty() {
        eprintln!("readlink: missing operand");
        return 1;
    }

    let mut exit_code = 0;
    let last = paths.len() - 1;

    for (idx, path) in paths.iter().enumerate() {
        let p = Path::new(path);
        let result = if canonicalize {
            resolve_path(p, false)
        } else if canonicalize_missing {
            resolve_path(p, true)
        } else {
            // Just read one symlink level
            match fs::read_link(p) {
                Ok(t) => Some(t),
                Err(e) => {
                    if !quiet {
                        eprintln!("readlink: {}: {}", path, e);
                    }
                    None
                }
            }
        };

        match result {
            Some(resolved) => {
                let s = resolved.display().to_string();
                if no_newline && idx == last {
                    print!("{}", s);
                } else {
                    println!("{}", s);
                }
            }
            None => exit_code = 1,
        }
    }
    exit_code
}
