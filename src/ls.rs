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

fn list_dir(
    dir: &Path,
    show_all: bool,
    long_format: bool,
    human_readable: bool,
    reverse: bool,
    sort_by_time: bool,
    one_per_line: bool,
) -> i32 {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("ls: cannot open directory '{}': {}", dir.display(), e);
            return 1;
        }
    };

    let mut items: Vec<(String, fs::Metadata)> = Vec::new();

    if show_all {
        if let Ok(meta) = dir.metadata() {
            items.push((".".to_string(), meta));
        }
        let parent = dir.parent().unwrap_or(dir);
        if let Ok(meta) = parent.metadata() {
            items.push(("..".to_string(), meta));
        }
    }

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !show_all && name.starts_with('.') {
            continue;
        }
        if let Ok(meta) = entry.metadata() {
            items.push((name, meta));
        }
    }

    if sort_by_time {
        items.sort_by(|a, b| {
            let ta = a.1.modified().unwrap_or(UNIX_EPOCH);
            let tb = b.1.modified().unwrap_or(UNIX_EPOCH);
            tb.cmp(&ta)
        });
    } else {
        items.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
    }

    if reverse {
        items.reverse();
    }

    if long_format {
        let total_blocks: u64 = items.iter().map(|(_, m)| m.blocks()).sum();
        println!("total {}", total_blocks / 2);
        for (name, meta) in &items {
            print_long_entry(name, meta, human_readable);
        }
    } else if one_per_line {
        for (name, meta) in &items {
            println!("{}", colorize_name(name, meta));
        }
    } else {
        let names: Vec<String> = items.iter().map(|(n, m)| colorize_name(n, m)).collect();
        print_columns(&names);
    }

    0
}

fn print_long_entry(name: &str, meta: &fs::Metadata, human_readable: bool) {
    let mode = meta.permissions().mode();
    let nlink = meta.nlink();
    let uid = meta.uid();
    let gid = meta.gid();
    let size = meta.len();
    let size_str = if human_readable {
        human_size(size)
    } else {
        size.to_string()
    };
    let mtime = meta
        .modified()
        .unwrap_or(UNIX_EPOCH)
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let time_str = format_time(mtime);
    let colored = colorize_name(name, meta);

    println!(
        "{} {:>3} {:>5} {:>5} {:>8} {} {}",
        format_mode(mode),
        nlink,
        uid,
        gid,
        size_str,
        time_str,
        colored
    );
}
