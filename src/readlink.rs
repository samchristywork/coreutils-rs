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

// Resolve path to canonical form. If allow_missing, resolve as far as possible.
fn resolve_path(path: &Path, allow_missing: bool) -> Option<std::path::PathBuf> {
    use std::path::PathBuf;

    // Build absolute path first
    let abs = if path.is_absolute() {
        path.to_path_buf()
    } else {
        match std::env::current_dir() {
            Ok(cwd) => cwd.join(path),
            Err(_) => return None,
        }
    };

    let mut resolved = PathBuf::from("/");
    let mut symlink_limit = 40;

    // Stack of remaining path segments (owned strings)
    let mut stack: Vec<PathBuf> = abs.components()
        .map(|c| PathBuf::from(c.as_os_str()))
        .collect();
    stack.reverse();

    while let Some(part) = stack.pop() {
        let s = part.to_string_lossy();
        if s == "/" {
            resolved = PathBuf::from("/");
        } else if s == "." {
            // skip
        } else if s == ".." {
            resolved.pop();
        } else {
            resolved.push(&*s);
            match fs::read_link(&resolved) {
                Ok(target) => {
                    symlink_limit -= 1;
                    if symlink_limit == 0 {
                        if !allow_missing { eprintln!("readlink: {}: Too many levels of symbolic links", resolved.display()); }
                        return None;
                    }
                    let expanded = if target.is_absolute() {
                        target
                    } else {
                        let mut base = resolved.clone();
                        base.pop();
                        base.join(target)
                    };
                    resolved.pop();
                    let mut extra: Vec<PathBuf> = expanded.components()
                        .map(|c| PathBuf::from(c.as_os_str()))
                        .collect();
                    extra.reverse();
                    stack.extend(extra);
                }
                Err(e) if e.raw_os_error() == Some(22) /* EINVAL = not a symlink */ => {
                    // Normal file/dir, keep as-is
                }
                Err(_) => {
                    if !allow_missing {
                        eprintln!("readlink: {}: No such file or directory", resolved.display());
                        return None;
                    }
                }
            }
        }
    }
    Some(resolved)
}
