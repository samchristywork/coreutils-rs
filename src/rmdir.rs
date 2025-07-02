use std::fs;
use std::path::Path;

pub fn run(args: &[String]) -> i32 {
    let mut parents = false;
    let mut verbose = false;
    let mut paths: Vec<String> = Vec::new();

    for arg in args {
        if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") {
            for ch in arg[1..].chars() {
                match ch {
                    'p' => parents = true,
                    'v' => verbose = true,
                    _ => {
                        eprintln!("rmdir: invalid option -- '{}'", ch);
                        return 1;
                    }
                }
            }
        } else {
            match arg.as_str() {
                "--parents" => parents = true,
                "--verbose" => verbose = true,
                a if a.starts_with('-') => {
                    eprintln!("rmdir: unrecognized option '{}'", a);
                    return 1;
                }
                _ => paths.push(arg.clone()),
            }
        }
    }

    if paths.is_empty() {
        eprintln!("rmdir: missing operand");
        return 1;
    }

    let mut exit_code = 0;
    for path in &paths {
        let p = Path::new(path);
        exit_code |= remove_dir(p, parents, verbose);
    }

    exit_code
}

fn remove_dir(p: &Path, parents: bool, verbose: bool) -> i32 {
    match fs::remove_dir(p) {
        Ok(()) => {
            if verbose {
                println!("rmdir: removing directory '{}'", p.display());
            }
            if parents {
                if let Some(parent) = p.parent() {
                    if parent != Path::new("") && parent != Path::new("/") {
                        // Ignore errors when removing parents - stop at first non-empty
                        let _ = remove_dir_silent(parent, verbose);
                    }
                }
            }
            0
        }
        Err(e) => {
            eprintln!("rmdir: failed to remove '{}': {}", p.display(), e);
            1
        }
    }
}

fn remove_dir_silent(p: &Path, verbose: bool) -> i32 {
    match fs::remove_dir(p) {
        Ok(()) => {
            if verbose {
                println!("rmdir: removing directory '{}'", p.display());
            }
            if let Some(parent) = p.parent() {
                if parent != Path::new("") && parent != Path::new("/") {
                    let _ = remove_dir_silent(parent, verbose);
                }
            }
            0
        }
        Err(_) => 1,
    }
}
