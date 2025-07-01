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
