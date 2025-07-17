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
