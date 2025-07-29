use std::fs;
use std::path::Path;

pub fn run(args: &[String]) -> i32 {
    let mut symbolic = false;
    let mut force = false;
    let mut no_dereference = false;
    let mut backup = false;
    let mut verbose = false;
    let mut paths: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            "-s" | "--symbolic"       => symbolic = true,
            "-f" | "--force"          => force = true,
            "-n" | "--no-dereference" => no_dereference = true,
            "-b" | "--backup"         => backup = true,
            "-v" | "--verbose"        => verbose = true,
            _ if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") => {
                for ch in arg[1..].chars() {
                    match ch {
                        's' => symbolic = true,
                        'f' => force = true,
                        'n' => no_dereference = true,
                        'b' => backup = true,
                        'v' => verbose = true,
                        _ => { eprintln!("ln: invalid option -- '{}'", ch); return 1; }
                    }
                }
            }
            _ if arg.starts_with('-') => { eprintln!("ln: unrecognized option '{}'", arg); return 1; }
            _ => paths.push(arg.to_string()),
        }
        i += 1;
    }

    if paths.is_empty() {
        eprintln!("ln: missing file operand");
        return 1;
    }
    if paths.len() == 1 {
        eprintln!("ln: missing destination file operand after '{}'", paths[0]);
        return 1;
    }

    let dest = Path::new(paths.last().unwrap());
    let sources = &paths[..paths.len() - 1];

    // Multiple sources require dest to be a directory
    if sources.len() > 1 && !dest.is_dir() {
        eprintln!("ln: target '{}': Not a directory", dest.display());
        return 1;
    }

    let mut exit_code = 0;
    for src in sources {
        let src_path = Path::new(src);
        let target = if dest.is_dir() && !(no_dereference && symbolic) {
            let name = src_path.file_name().unwrap_or(src_path.as_os_str());
            dest.join(name)
        } else {
            dest.to_path_buf()
        };

        if target.exists() || target.symlink_metadata().is_ok() {
            if backup {
                let bak = target.with_extension("~");
                if let Err(e) = fs::rename(&target, &bak) {
                    eprintln!("ln: cannot backup '{}': {}", target.display(), e);
                    exit_code = 1;
                    continue;
                }
            } else if force {
                if let Err(e) = fs::remove_file(&target) {
                    eprintln!("ln: cannot remove '{}': {}", target.display(), e);
                    exit_code = 1;
                    continue;
                }
            } else {
                eprintln!("ln: failed to create link '{}': File exists", target.display());
                exit_code = 1;
                continue;
            }
        }

        let result = if symbolic {
            std::os::unix::fs::symlink(src_path, &target)
        } else {
            fs::hard_link(src_path, &target)
        };

        match result {
            Ok(()) => {
                if verbose {
                    println!("'{}' -> '{}'", target.display(), src);
                }
            }
            Err(e) => {
                eprintln!("ln: failed to create {} link '{}' -> '{}': {}",
                    if symbolic { "symbolic" } else { "hard" },
                    target.display(), src, e);
                exit_code = 1;
            }
        }
    }
    exit_code
}
