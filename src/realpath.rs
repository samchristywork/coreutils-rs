use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::fs;

pub fn run(args: &[String]) -> i32 {
    let mut logical = false;
    let mut no_symlinks = false;
    let mut quiet = false;
    let mut relative_to: Option<String> = None;
    let mut relative_base: Option<String> = None;
    let mut zero = false;
    let mut missing_ok = false;
    let mut paths: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            "-L" | "--logical"           => logical = true,
            "-P" | "--physical"          => logical = false,
            "-q" | "--quiet"             => quiet = true,
            "-z" | "--zero"              => zero = true,
            "-m" | "--canonicalize-missing" => missing_ok = true,
            "-s" | "--strip" | "--no-symlinks" => no_symlinks = true,
            "--relative-to" => {
                i += 1;
                if i >= args.len() { eprintln!("realpath: option requires an argument -- 'relative-to'"); return 1; }
                relative_to = Some(args[i].clone());
            }
            "--relative-base" => {
                i += 1;
                if i >= args.len() { eprintln!("realpath: option requires an argument -- 'relative-base'"); return 1; }
                relative_base = Some(args[i].clone());
            }
            _ if arg.starts_with("--relative-to=") => {
                relative_to = Some(arg["--relative-to=".len()..].to_string());
            }
            _ if arg.starts_with("--relative-base=") => {
                relative_base = Some(arg["--relative-base=".len()..].to_string());
            }
            _ if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") => {
                for ch in arg[1..].chars() {
                    match ch {
                        'L' => logical = true,
                        'P' => logical = false,
                        'q' => quiet = true,
                        'z' => zero = true,
                        'm' => missing_ok = true,
                        's' => no_symlinks = true,
                        _ => { eprintln!("realpath: invalid option -- '{}'", ch); return 1; }
                    }
                }
            }
            _ if arg.starts_with('-') => { eprintln!("realpath: unrecognized option '{}'", arg); return 1; }
            _ => paths.push(arg.to_string()),
        }
        i += 1;
    }

    if paths.is_empty() {
        eprintln!("realpath: missing operand");
        return 1;
    }

    // Resolve --relative-to and --relative-base against cwd if needed
    let rel_to = relative_to.as_deref().and_then(|p| resolve(Path::new(p), logical, no_symlinks, true));
    let rel_base = relative_base.as_deref().and_then(|p| resolve(Path::new(p), logical, no_symlinks, true));

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let mut exit_code = 0;

    for path in &paths {
        let p = Path::new(path);
        let resolved = resolve(p, logical, no_symlinks, missing_ok);

        let resolved = match resolved {
            Some(r) => r,
            None => {
                if !quiet { eprintln!("realpath: {}: No such file or directory", path); }
                exit_code = 1;
                continue;
            }
        };

        let display = if let Some(ref rt) = rel_to {
            match make_relative(&resolved, rt) {
                Some(r) => r,
                None => resolved,
            }
        } else if let Some(ref rb) = rel_base {
            // Only make relative if resolved is under rel_base
            if resolved.starts_with(rb) {
                make_relative(&resolved, rb).unwrap_or(resolved)
            } else {
                resolved
            }
        } else {
            resolved
        };

        let s = display.display().to_string();
        if zero {
            let _ = write!(out, "{}\0", s);
        } else {
            let _ = writeln!(out, "{}", s);
        }
    }
    exit_code
}

fn resolve(path: &Path, logical: bool, no_symlinks: bool, missing_ok: bool) -> Option<PathBuf> {
    if no_symlinks {
        // Just make absolute without resolving symlinks
        return Some(make_absolute(path));
    }

    if logical {
        // Resolve ./ and ../ but don't follow symlinks
        return Some(normalize(make_absolute(path)));
    }

    // Physical: resolve all symlinks
    canonicalize_path(path, missing_ok)
}

fn make_absolute(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")).join(path)
    }
}
