use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::fs;

pub fn run(args: &[String]) -> i32 {
    let mut recursive = false;
    let mut verbose = false;
    let mut changes = false;
    let mut mode_str: Option<String> = None;
    let mut paths: Vec<String> = Vec::new();

    for arg in args {
        if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") {
            for ch in arg[1..].chars() {
                match ch {
                    'R' => recursive = true,
                    'v' => verbose = true,
                    'c' => changes = true,
                    _ => { eprintln!("chmod: invalid option -- '{}'", ch); return 1; }
                }
            }
        } else {
            match arg.as_str() {
                "--recursive" => recursive = true,
                "--verbose"   => verbose = true,
                "--changes"   => changes = true,
                a if a.starts_with('-') => { eprintln!("chmod: unrecognized option '{}'", a); return 1; }
                _ => {
                    if mode_str.is_none() {
                        mode_str = Some(arg.clone());
                    } else {
                        paths.push(arg.clone());
                    }
                }
            }
        }
    }

    let mode_str = match mode_str {
        Some(m) => m,
        None => { eprintln!("chmod: missing operand"); return 1; }
    };

    if paths.is_empty() {
        eprintln!("chmod: missing operand after '{}'", mode_str);
        return 1;
    }

    let mut exit_code = 0;
    for path in &paths {
        exit_code |= chmod_path(Path::new(path), &mode_str, recursive, verbose, changes);
    }
    exit_code
}

fn chmod_path(path: &Path, mode_str: &str, recursive: bool, verbose: bool, changes: bool) -> i32 {
    let meta = match fs::metadata(path) {
        Ok(m) => m,
        Err(e) => { eprintln!("chmod: cannot access '{}': {}", path.display(), e); return 1; }
    };

    let old_mode = meta.permissions().mode();
    let new_mode = match apply_mode(old_mode, mode_str) {
        Some(m) => m,
        None => { eprintln!("chmod: invalid mode: '{}'", mode_str); return 1; }
    };

    // For directories under -R: recurse first, then apply (so we don't lose execute access)
    if recursive && meta.is_dir() {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                chmod_path(&entry.path(), mode_str, recursive, verbose, changes);
            }
        }
    }

    if new_mode != old_mode {
        let perms = fs::Permissions::from_mode(new_mode);
        if let Err(e) = fs::set_permissions(path, perms) {
            eprintln!("chmod: cannot change permissions of '{}': {}", path.display(), e);
            return 1;
        }
        if verbose || changes {
            println!("mode of '{}' changed from {:04o} to {:04o}", path.display(), old_mode & 0o7777, new_mode & 0o7777);
        }
    } else if verbose {
        println!("mode of '{}' retained as {:04o}", path.display(), old_mode & 0o7777);
    }

    0
}
