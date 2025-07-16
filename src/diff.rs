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

fn read_dir_names(dir: &str) -> Vec<String> {
    fs::read_dir(dir).ok().into_iter().flatten().flatten()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect()
}

fn normalize(s: &str, opts: &Opts) -> String {
    let mut s = s.to_string();
    if opts.ignore_case { s = s.to_lowercase(); }
    if opts.ignore_all_space {
        s.retain(|c| c != ' ' && c != '\t');
    } else if opts.ignore_space_change {
        let mut out = String::new();
        let mut prev = false;
        for ch in s.chars() {
            if ch == ' ' || ch == '\t' {
                if !prev { out.push(' '); }
                prev = true;
            } else {
                out.push(ch); prev = false;
            }
        }
        s = out.trim().to_string();
    }
    if opts.ignore_blank_lines && s.trim().is_empty() {
        s = "\x00blank\x00".to_string();
    }
    s
}

#[derive(Clone, PartialEq)]
enum Kind { Keep, Delete, Insert }

struct Edit {
    kind: Kind,
    idx: usize, // into a (Delete/Keep) or b (Insert)
}

fn compute_diff(a: &[String], b: &[String]) -> Vec<Edit> {
    let n = a.len();
    let m = b.len();

    // LCS via DP, then build edits from the traceback
    // dp[i][j] = length of LCS of a[..i] and b[..j]
    let mut dp = vec![vec![0u32; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            dp[i][j] = if a[i] == b[j] {
                dp[i + 1][j + 1] + 1
            } else {
                dp[i + 1][j].max(dp[i][j + 1])
            };
        }
    }

    let mut edits = Vec::new();
    let mut i = 0;
    let mut j = 0;
    while i < n || j < m {
        if i < n && j < m && a[i] == b[j] {
            edits.push(Edit { kind: Kind::Keep, idx: i });
            i += 1; j += 1;
        } else if j < m && (i >= n || dp[i][j + 1] > dp[i + 1][j]) {
            edits.push(Edit { kind: Kind::Insert, idx: j });
            j += 1;
        } else {
            edits.push(Edit { kind: Kind::Delete, idx: i });
            i += 1;
        }
    }
    edits
}

fn print_normal<W: Write>(a: &[String], b: &[String], edits: &[Edit], out: &mut W) {
    let n = edits.len();
    let mut i = 0;
    while i < n {
        if edits[i].kind == Kind::Keep { i += 1; continue; }

        let start = i;
        while i < n && edits[i].kind != Kind::Keep { i += 1; }

        let dels: Vec<usize> = edits[start..i].iter().filter(|e| e.kind == Kind::Delete).map(|e| e.idx).collect();
        let ins: Vec<usize>  = edits[start..i].iter().filter(|e| e.kind == Kind::Insert).map(|e| e.idx).collect();

        let left = range_str(dels.first().copied(), dels.last().copied());
        let right = range_str(ins.first().copied(), ins.last().copied());

        let op = match (!dels.is_empty(), !ins.is_empty()) {
            (true, true)  => 'c',
            (true, false) => 'd',
            (false, true) => 'a',
            (false, false) => continue,
        };

        // For 'a', left is the line after which insertion happens
        let left_str = if op == 'a' {
            format!("{}", dels.first().map(|&x| x).unwrap_or(ins[0].saturating_sub(1)) + 1)
        } else { left };
        // For 'd', right is the line before which deletion would go
        let right_str = if op == 'd' {
            format!("{}", ins.first().map(|&x| x + 1).unwrap_or(dels.last().map(|&x| x + 1).unwrap_or(0)))
        } else { right };

        let _ = writeln!(out, "{}{}{}", left_str, op, right_str);
        for &x in &dels { let _ = writeln!(out, "< {}", a[x]); }
        if !dels.is_empty() && !ins.is_empty() { let _ = writeln!(out, "---"); }
        for &y in &ins  { let _ = writeln!(out, "> {}", b[y]); }
    }
}
