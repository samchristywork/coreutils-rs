use std::fs;
use std::io;
use std::path::Path;

pub fn run(args: &[String]) -> i32 {
    let mut recursive = false;
    let mut preserve = false;
    let mut no_clobber = false;
    let mut verbose = false;
    let mut force = false;
    let mut paths: Vec<String> = Vec::new();

    for arg in args {
        if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") {
            for ch in arg[1..].chars() {
                match ch {
                    'r' | 'R' => recursive = true,
                    'p' => preserve = true,
                    'n' => no_clobber = true,
                    'v' => verbose = true,
                    'f' => force = true,
                    _ => {
                        eprintln!("cp: invalid option -- '{}'", ch);
                        return 1;
                    }
                }
            }
        } else {
            match arg.as_str() {
                "--recursive" => recursive = true,
                "--preserve" => preserve = true,
                "--no-clobber" => no_clobber = true,
                "--verbose" => verbose = true,
                "--force" => force = true,
                a if a.starts_with('-') => {
                    eprintln!("cp: unrecognized option '{}'", a);
                    return 1;
                }
                _ => paths.push(arg.clone()),
            }
        }
    }

    if paths.len() < 2 {
        eprintln!("cp: missing file operand");
        eprintln!("Usage: cp [OPTION]... SOURCE... DEST");
        return 1;
    }

    let dest = paths.last().unwrap().clone();
    let sources = &paths[..paths.len() - 1];
    let dest_path = Path::new(&dest);

    // Multiple sources require dest to be a directory
    if sources.len() > 1 && !dest_path.is_dir() {
        eprintln!("cp: target '{}' is not a directory", dest);
        return 1;
    }

    let mut exit_code = 0;
    for src in sources {
        let src_path = Path::new(src);
        if !src_path.exists() {
            eprintln!("cp: cannot stat '{}': No such file or directory", src);
            exit_code = 1;
            continue;
        }

        let effective_dest = if dest_path.is_dir() {
            let name = src_path.file_name().unwrap_or_default();
            dest_path.join(name)
        } else {
            dest_path.to_path_buf()
        };

        if src_path.is_dir() {
            if !recursive {
                eprintln!("cp: -r not specified; omitting directory '{}'", src);
                exit_code = 1;
                continue;
            }
            exit_code |= copy_dir(src_path, &effective_dest, preserve, no_clobber, verbose, force);
        } else {
            exit_code |= copy_file(src_path, &effective_dest, preserve, no_clobber, verbose, force);
        }
    }

    exit_code
}

fn copy_file(src: &Path, dest: &Path, preserve: bool, no_clobber: bool, verbose: bool, force: bool) -> i32 {
    if dest.exists() {
        if no_clobber {
            return 0;
        }
        if force {
            let _ = fs::remove_file(dest);
        }
    }

    match fs::copy(src, dest) {
        Ok(_) => {
            if preserve {
                if let Ok(src_meta) = src.metadata() {
                    let _ = fs::set_permissions(dest, src_meta.permissions());
                    if let Ok(dest_file) = fs::File::open(dest) {
                        let _ = set_times(&dest_file, &src_meta);
                    }
                }
            }
            if verbose {
                println!("'{}' -> '{}'", src.display(), dest.display());
            }
            0
        }
        Err(e) => {
            eprintln!("cp: cannot copy '{}' to '{}': {}", src.display(), dest.display(), e);
            1
        }
    }
}
