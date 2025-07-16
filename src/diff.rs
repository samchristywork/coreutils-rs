use std::fs;
use std::io::{self, Read, Write};

pub fn run(args: &[String]) -> i32 {
    let mut unified: Option<usize> = None;
    let mut ignore_case = false;
    let mut ignore_blank_lines = false;
    let mut ignore_space_change = false;
    let mut ignore_all_space = false;
    let mut recursive = false;
    let mut paths: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            "-u" | "--unified"              => unified = Some(3),
            "-i" | "--ignore-case"          => ignore_case = true,
            "-B" | "--ignore-blank-lines"   => ignore_blank_lines = true,
            "-b" | "--ignore-space-change"  => ignore_space_change = true,
            "-w" | "--ignore-all-space"     => ignore_all_space = true,
            "-r" | "--recursive"            => recursive = true,
            _ if arg.starts_with("-U") => {
                let rest = &arg[2..];
                let val = if rest.is_empty() {
                    i += 1;
                    if i >= args.len() { eprintln!("diff: option requires an argument -- 'U'"); return 2; }
                    args[i].as_str()
                } else { rest };
                match val.parse() {
                    Ok(n) => unified = Some(n),
                    Err(_) => { eprintln!("diff: invalid context length '{}'", val); return 2; }
                }
            }
            _ if arg.starts_with("--unified=") => {
                match arg["--unified=".len()..].parse() {
                    Ok(n) => unified = Some(n),
                    Err(_) => { eprintln!("diff: invalid context length"); return 2; }
                }
            }
            _ if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") => {
                for ch in arg[1..].chars() {
                    match ch {
                        'u' => unified = Some(3),
                        'i' => ignore_case = true,
                        'B' => ignore_blank_lines = true,
                        'b' => ignore_space_change = true,
                        'w' => ignore_all_space = true,
                        'r' => recursive = true,
                        _ => { eprintln!("diff: invalid option -- '{}'", ch); return 2; }
                    }
                }
            }
            _ if arg.starts_with('-') => { eprintln!("diff: unrecognized option '{}'", arg); return 2; }
            _ => paths.push(args[i].clone()),
        }
        i += 1;
    }

    if paths.len() < 2 {
        eprintln!("diff: missing operand");
        return 2;
    }

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let opts = Opts { unified, ignore_case, ignore_blank_lines, ignore_space_change, ignore_all_space };
    diff_paths(&paths[0], &paths[1], recursive, &opts, &mut out)
}

struct Opts {
    unified: Option<usize>,
    ignore_case: bool,
    ignore_blank_lines: bool,
    ignore_space_change: bool,
    ignore_all_space: bool,
}

fn diff_paths<W: Write>(path1: &str, path2: &str, recursive: bool, opts: &Opts, out: &mut W) -> i32 {
    let is_dir1 = fs::metadata(path1).map(|m| m.is_dir()).unwrap_or(false);
    let is_dir2 = fs::metadata(path2).map(|m| m.is_dir()).unwrap_or(false);

    if is_dir1 && is_dir2 {
        if !recursive {
            eprintln!("diff: {}: Is a directory", path1);
            return 2;
        }
        return diff_dirs(path1, path2, opts, out);
    }

    let lines1 = match read_lines(path1) {
        Ok(l) => l,
        Err(e) => { eprintln!("diff: {}: {}", path1, e); return 2; }
    };
    let lines2 = match read_lines(path2) {
        Ok(l) => l,
        Err(e) => { eprintln!("diff: {}: {}", path2, e); return 2; }
    };

    let cmp1: Vec<String> = lines1.iter().map(|l| normalize(l, opts)).collect();
    let cmp2: Vec<String> = lines2.iter().map(|l| normalize(l, opts)).collect();

    let edits = compute_diff(&cmp1, &cmp2);

    if edits.iter().all(|e| e.kind == Kind::Keep) {
        return 0;
    }

    if let Some(ctx) = opts.unified {
        print_unified(&lines1, &lines2, &edits, path1, path2, ctx, out);
    } else {
        print_normal(&lines1, &lines2, &edits, out);
    }

    1
}

fn diff_dirs<W: Write>(dir1: &str, dir2: &str, opts: &Opts, out: &mut W) -> i32 {
    let mut names1 = read_dir_names(dir1);
    let mut names2 = read_dir_names(dir2);
    names1.sort();
    names2.sort();

    let mut exit_code = 0;
    let mut i = 0;
    let mut j = 0;
    while i < names1.len() || j < names2.len() {
        let ord = match (names1.get(i), names2.get(j)) {
            (None, _) => std::cmp::Ordering::Greater,
            (_, None) => std::cmp::Ordering::Less,
            (Some(a), Some(b)) => a.cmp(b),
        };
        match ord {
            std::cmp::Ordering::Less => {
                let _ = writeln!(out, "Only in {}: {}", dir1, names1[i]);
                i += 1; exit_code = 1;
            }
            std::cmp::Ordering::Greater => {
                let _ = writeln!(out, "Only in {}: {}", dir2, names2[j]);
                j += 1; exit_code = 1;
            }
            std::cmp::Ordering::Equal => {
                let p1 = format!("{}/{}", dir1, names1[i]);
                let p2 = format!("{}/{}", dir2, names2[j]);
                exit_code |= diff_paths(&p1, &p2, true, opts, out);
                i += 1; j += 1;
            }
        }
    }
    exit_code
}
