use std::fs;
use std::path::Path;

pub fn run(args: &[String]) -> i32 {
    let mut no_clobber = false;
    let mut verbose = false;
    let mut force = false;
    let mut paths: Vec<String> = Vec::new();

    for arg in args {
        if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") {
            for ch in arg[1..].chars() {
                match ch {
                    'n' => no_clobber = true,
                    'v' => verbose = true,
                    'f' => force = true,
                    _ => {
                        eprintln!("mv: invalid option -- '{}'", ch);
                        return 1;
                    }
                }
            }
        } else {
            match arg.as_str() {
                "--no-clobber" => no_clobber = true,
                "--verbose" => verbose = true,
                "--force" => force = true,
                a if a.starts_with('-') => {
                    eprintln!("mv: unrecognized option '{}'", a);
                    return 1;
                }
                _ => paths.push(arg.clone()),
            }
        }
    }

    if paths.len() < 2 {
        eprintln!("mv: missing file operand");
        eprintln!("Usage: mv [OPTION]... SOURCE... DEST");
        return 1;
    }

    let dest = paths.last().unwrap().clone();
    let sources = &paths[..paths.len() - 1];
    let dest_path = Path::new(&dest);

    if sources.len() > 1 && !dest_path.is_dir() {
        eprintln!("mv: target '{}' is not a directory", dest);
        return 1;
    }

    let mut exit_code = 0;
    for src in sources {
        let src_path = Path::new(src);
        if !src_path.exists() {
            eprintln!("mv: cannot stat '{}': No such file or directory", src);
            exit_code = 1;
            continue;
        }

        let effective_dest = if dest_path.is_dir() {
            let name = src_path.file_name().unwrap_or_default();
            dest_path.join(name)
        } else {
            dest_path.to_path_buf()
        };

        if effective_dest.exists() {
            if no_clobber {
                continue;
            }
            if force {
                if effective_dest.is_dir() {
                    let _ = fs::remove_dir_all(&effective_dest);
                } else {
                    let _ = fs::remove_file(&effective_dest);
                }
            }
        }

        // Try atomic rename first; fall back to copy+delete for cross-device moves
        match fs::rename(src_path, &effective_dest) {
            Ok(()) => {
                if verbose {
                    println!("'{}' -> '{}'", src_path.display(), effective_dest.display());
                }
            }
            Err(e) if is_cross_device(&e) => {
                let code = if src_path.is_dir() {
                    copy_dir_recursive(src_path, &effective_dest)
                } else {
                    copy_then_remove_file(src_path, &effective_dest)
                };
                if code != 0 {
                    exit_code = 1;
                    continue;
                }
                if verbose {
                    println!("'{}' -> '{}'", src_path.display(), effective_dest.display());
                }
            }
            Err(e) => {
                eprintln!("mv: cannot move '{}' to '{}': {}", src, effective_dest.display(), e);
                exit_code = 1;
            }
        }
    }

    exit_code
}
