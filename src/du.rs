use std::collections::HashSet;
use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::MetadataExt;
use std::path::Path;

pub fn run(args: &[String]) -> i32 {
    let mut all = false;
    let mut summarize = false;
    let mut human = false;
    let mut block_size: u64 = 1024;
    let mut max_depth: Option<usize> = None;
    let mut one_fs = false;
    let mut paths: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            "-a" | "--all"             => all = true,
            "-s" | "--summarize"       => summarize = true,
            "-h" | "--human-readable"  => human = true,
            "-k"                       => block_size = 1024,
            "-m"                       => block_size = 1024 * 1024,
            "-x" | "--one-file-system" => one_fs = true,
            "--max-depth" => {
                i += 1;
                if i >= args.len() { eprintln!("du: option requires an argument -- 'max-depth'"); return 1; }
                match args[i].parse() {
                    Ok(n) => max_depth = Some(n),
                    Err(_) => { eprintln!("du: invalid max depth '{}'", args[i]); return 1; }
                }
            }
            _ if arg.starts_with("--max-depth=") => {
                match arg["--max-depth=".len()..].parse() {
                    Ok(n) => max_depth = Some(n),
                    Err(_) => { eprintln!("du: invalid max depth"); return 1; }
                }
            }
            _ if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") => {
                let mut chars = arg[1..].chars().peekable();
                while let Some(ch) = chars.next() {
                    match ch {
                        'a' => all = true,
                        's' => summarize = true,
                        'h' => human = true,
                        'k' => block_size = 1024,
                        'm' => block_size = 1024 * 1024,
                        'x' => one_fs = true,
                        'd' => {
                            let rest: String = chars.collect();
                            let val = if rest.is_empty() {
                                i += 1;
                                if i >= args.len() { eprintln!("du: option requires an argument -- 'd'"); return 1; }
                                args[i].clone()
                            } else { rest };
                            match val.parse() {
                                Ok(n) => max_depth = Some(n),
                                Err(_) => { eprintln!("du: invalid max depth '{}'", val); return 1; }
                            }
                            break;
                        }
                        _ => { eprintln!("du: invalid option -- '{}'", ch); return 1; }
                    }
                }
            }
            _ if arg.starts_with('-') => { eprintln!("du: unrecognized option '{}'", arg); return 1; }
            _ => paths.push(args[i].clone()),
        }
        i += 1;
    }

    if paths.is_empty() { paths.push(".".to_string()); }

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let mut seen_inodes: HashSet<(u64, u64)> = HashSet::new();
    let mut exit_code = 0;

    for path in &paths {
        let p = Path::new(path);
        match du_path(p, 0, all, summarize, human, block_size, max_depth, one_fs, &mut seen_inodes, &mut out) {
            Ok(_) => {}
            Err(_) => exit_code = 1,
        }
    }
    exit_code
}

#[allow(clippy::too_many_arguments)]
fn du_path<W: Write>(
    path: &Path,
    depth: usize,
    all: bool,
    summarize: bool,
    human: bool,
    block_size: u64,
    max_depth: Option<usize>,
    one_fs: bool,
    seen: &mut HashSet<(u64, u64)>,
    out: &mut W,
) -> Result<u64, ()> {
    let meta = match fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("du: cannot access '{}': {}", path.display(), e);
            return Err(());
        }
    };

    let dev = meta.dev();
    let ino = meta.ino();

    // Skip hard-linked duplicates for non-directories
    if !meta.is_dir() && meta.nlink() > 1 && !seen.insert((dev, ino)) {
        return Ok(0);
    }

    // 512-byte blocks from stat, convert to our block_size
    let file_blocks = meta.blocks() * 512;

    if !meta.is_dir() {
        let display_blocks = file_blocks.div_ceil(block_size);
        if all && !summarize && max_depth.is_none_or(|d| depth <= d) {
            print_entry(display_blocks, path, human, block_size, out);
        }
        return Ok(file_blocks);
    }

    let root_dev = dev;
    let mut total = file_blocks;

    let entries = match fs::read_dir(path) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("du: cannot read directory '{}': {}", path.display(), e);
            return Err(());
        }
    };

    for entry in entries.flatten() {
        let child = entry.path();
        let child_meta = match fs::symlink_metadata(&child) {
            Ok(m) => m,
            Err(_) => continue,
        };

        if one_fs && child_meta.dev() != root_dev {
            continue;
        }

        if let Ok(n) = du_path(&child, depth + 1, all, summarize, human, block_size, max_depth, one_fs, seen, out) {
            total += n;
        }
    }

    let display_blocks = total.div_ceil(block_size);
    let show = if summarize {
        depth == 0
    } else {
        max_depth.is_none_or(|d| depth <= d)
    };

    if show {
        print_entry(display_blocks, path, human, block_size, out);
    }

    Ok(total)
}

fn print_entry<W: Write>(blocks: u64, path: &Path, human: bool, block_size: u64, out: &mut W) {
    if human {
        let bytes = blocks * block_size;
        let _ = writeln!(out, "{}\t{}", human_size(bytes), path.display());
    } else {
        let _ = writeln!(out, "{}\t{}", blocks, path.display());
    }
}

fn human_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "K", "M", "G", "T", "P"];
    let mut size = bytes as f64;
    let mut idx = 0;
    while size >= 1024.0 && idx + 1 < UNITS.len() { size /= 1024.0; idx += 1; }
    if idx == 0 { format!("{}", bytes) }
    else if size < 10.0 { format!("{:.1}{}", size, UNITS[idx]) }
    else { format!("{:.0}{}", size, UNITS[idx]) }
}
