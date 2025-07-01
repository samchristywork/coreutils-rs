use std::fs;
use std::path::Path;

pub fn run(args: &[String]) -> i32 {
    let mut parents = false;
    let mut verbose = false;
    let mut mode: Option<u32> = None;
    let mut paths: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-p" | "--parents" => parents = true,
            "-v" | "--verbose" => verbose = true,
            "-m" | "--mode" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("mkdir: option requires an argument -- 'm'");
                    return 1;
                }
                match parse_mode(&args[i]) {
                    Some(m) => mode = Some(m),
                    None => {
                        eprintln!("mkdir: invalid mode '{}'", args[i]);
                        return 1;
                    }
                }
            }
            arg if arg.starts_with("--mode=") => {
                match parse_mode(&arg["--mode=".len()..]) {
                    Some(m) => mode = Some(m),
                    None => {
                        eprintln!("mkdir: invalid mode '{}'", &arg["--mode=".len()..]);
                        return 1;
                    }
                }
            }
            arg if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") => {
                let mut chars = arg[1..].chars().peekable();
                while let Some(ch) = chars.next() {
                    match ch {
                        'p' => parents = true,
                        'v' => verbose = true,
                        'm' => {
                            let rest: String = chars.collect();
                            let mode_str = if rest.is_empty() {
                                i += 1;
                                if i >= args.len() {
                                    eprintln!("mkdir: option requires an argument -- 'm'");
                                    return 1;
                                }
                                args[i].clone()
                            } else {
                                rest
                            };
                            match parse_mode(&mode_str) {
                                Some(m) => mode = Some(m),
                                None => {
                                    eprintln!("mkdir: invalid mode '{}'", mode_str);
                                    return 1;
                                }
                            }
                            break;
                        }
                        _ => {
                            eprintln!("mkdir: invalid option -- '{}'", ch);
                            return 1;
                        }
                    }
                }
            }
            arg if arg.starts_with('-') => {
                eprintln!("mkdir: unrecognized option '{}'", arg);
                return 1;
            }
            _ => paths.push(args[i].clone()),
        }
        i += 1;
    }

    if paths.is_empty() {
        eprintln!("mkdir: missing operand");
        return 1;
    }

    let mut exit_code = 0;
    for path in &paths {
        let p = Path::new(path);
        exit_code |= make_dir(p, parents, verbose, mode);
    }

    exit_code
}

fn make_dir(p: &Path, parents: bool, verbose: bool, mode: Option<u32>) -> i32 {
    if parents {
        // With -p, create intermediate dirs and don't error if already exists
        match fs::create_dir_all(p) {
            Ok(()) => {}
            Err(e) => {
                eprintln!("mkdir: cannot create directory '{}': {}", p.display(), e);
                return 1;
            }
        }
        if verbose {
            // Print each component that was created
            println!("mkdir: created directory '{}'", p.display());
        }
    } else {
        match fs::create_dir(p) {
            Ok(()) => {
                if verbose {
                    println!("mkdir: created directory '{}'", p.display());
                }
            }
            Err(e) => {
                eprintln!("mkdir: cannot create directory '{}': {}", p.display(), e);
                return 1;
            }
        }
    }

    if let Some(m) = mode {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(m);
            if let Err(e) = fs::set_permissions(p, perms) {
                eprintln!("mkdir: cannot set permissions on '{}': {}", p.display(), e);
                return 1;
            }
        }
    }

    0
}

fn parse_mode(s: &str) -> Option<u32> {
    // Accept octal (e.g. 755, 0755) or symbolic modes are not supported
    let s = s.trim_start_matches('0');
    let s = if s.is_empty() { "0" } else { s };
    u32::from_str_radix(s, 8).ok()
}
