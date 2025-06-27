use std::fs;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::time::UNIX_EPOCH;

pub fn run(args: &[String]) -> i32 {
    let mut show_all = false;
    let mut long_format = false;
    let mut human_readable = false;
    let mut reverse = false;
    let mut sort_by_time = false;
    let mut one_per_line = false;
    let mut paths: Vec<String> = Vec::new();

    for arg in args {
        if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") {
            for ch in arg[1..].chars() {
                match ch {
                    'a' => show_all = true,
                    'l' => long_format = true,
                    'h' => human_readable = true,
                    'r' => reverse = true,
                    't' => sort_by_time = true,
                    '1' => one_per_line = true,
                    _ => {
                        eprintln!("ls: invalid option -- '{}'", ch);
                        return 1;
                    }
                }
            }
        } else if arg == "--all" {
            show_all = true;
        } else if arg == "--human-readable" {
            human_readable = true;
        } else if arg == "--reverse" {
            reverse = true;
        } else if arg.starts_with('-') {
            eprintln!("ls: unrecognized option '{}'", arg);
            return 1;
        } else {
            paths.push(arg.clone());
        }
    }

    if paths.is_empty() {
        paths.push(".".to_string());
    }

    let multiple_dirs = paths.len() > 1;
    let mut exit_code = 0;

    for (idx, path) in paths.iter().enumerate() {
        let p = Path::new(path);
        if !p.exists() {
            eprintln!("ls: cannot access '{}': No such file or directory", path);
            exit_code = 1;
            continue;
        }

        if multiple_dirs {
            if idx > 0 {
                println!();
            }
            println!("{}:", path);
        }

        if p.is_dir() {
            exit_code |= list_dir(
                p,
                show_all,
                long_format,
                human_readable,
                reverse,
                sort_by_time,
                one_per_line || long_format,
            );
        } else {
            match p.metadata() {
                Ok(meta) => {
                    if long_format {
                        print_long_entry(path, &meta, human_readable);
                    } else {
                        println!("{}", colorize_name(path, &meta));
                    }
                }
                Err(e) => {
                    eprintln!("ls: cannot access '{}': {}", path, e);
                    exit_code = 1;
                }
            }
        }
    }

    exit_code
}
