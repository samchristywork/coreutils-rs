use std::fs;
use std::path::Path;

pub fn run(args: &[String]) -> i32 {
    let mut recursive = false;
    let mut force = false;
    let mut verbose = false;
    let mut paths: Vec<String> = Vec::new();

    for arg in args {
        if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") {
            for ch in arg[1..].chars() {
                match ch {
                    'r' | 'R' => recursive = true,
                    'f' => force = true,
                    'v' => verbose = true,
                    _ => {
                        eprintln!("rm: invalid option -- '{}'", ch);
                        return 1;
                    }
                }
            }
        } else {
            match arg.as_str() {
                "--recursive" => recursive = true,
                "--force" => force = true,
                "--verbose" => verbose = true,
                a if a.starts_with('-') => {
                    eprintln!("rm: unrecognized option '{}'", a);
                    return 1;
                }
                _ => paths.push(arg.clone()),
            }
        }
    }

    if paths.is_empty() {
        if !force {
            eprintln!("rm: missing operand");
        }
        return 0;
    }

    let mut exit_code = 0;
    for path in &paths {
        let p = Path::new(path);

        if !p.exists() {
            if !force {
                eprintln!("rm: cannot remove '{}': No such file or directory", path);
                exit_code = 1;
            }
            continue;
        }

        if p.is_dir() {
            if !recursive {
                eprintln!("rm: cannot remove '{}': Is a directory", path);
                exit_code = 1;
                continue;
            }
            exit_code |= remove_dir(p, force, verbose);
        } else {
            exit_code |= remove_file(p, force, verbose);
        }
    }

    exit_code
}

fn remove_file(p: &Path, force: bool, verbose: bool) -> i32 {
    match fs::remove_file(p) {
        Ok(()) => {
            if verbose {
                println!("removed '{}'", p.display());
            }
            0
        }
        Err(e) => {
            if !force {
                eprintln!("rm: cannot remove '{}': {}", p.display(), e);
            }
            if force { 0 } else { 1 }
        }
    }
}

fn remove_dir(p: &Path, force: bool, verbose: bool) -> i32 {
    let entries = match fs::read_dir(p) {
        Ok(e) => e,
        Err(e) => {
            if !force {
                eprintln!("rm: cannot read directory '{}': {}", p.display(), e);
            }
            return if force { 0 } else { 1 };
        }
    };

    let mut exit_code = 0;
    for entry in entries.flatten() {
        let child = entry.path();
        if child.is_dir() {
            exit_code |= remove_dir(&child, force, verbose);
        } else {
            exit_code |= remove_file(&child, force, verbose);
        }
    }

    if exit_code != 0 {
        return exit_code;
    }

    match fs::remove_dir(p) {
        Ok(()) => {
            if verbose {
                println!("removed directory '{}'", p.display());
            }
            0
        }
        Err(e) => {
            if !force {
                eprintln!("rm: cannot remove directory '{}': {}", p.display(), e);
            }
            if force { 0 } else { 1 }
        }
    }
}
