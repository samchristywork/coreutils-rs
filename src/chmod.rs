use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::fs;

pub fn run(args: &[String]) -> i32 {
    let mut recursive = false;
    let mut verbose = false;
    let mut changes = false;
    let mut mode_str: Option<String> = None;
    let mut paths: Vec<String> = Vec::new();

    for arg in args {
        if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") {
            for ch in arg[1..].chars() {
                match ch {
                    'R' => recursive = true,
                    'v' => verbose = true,
                    'c' => changes = true,
                    _ => { eprintln!("chmod: invalid option -- '{}'", ch); return 1; }
                }
            }
        } else {
            match arg.as_str() {
                "--recursive" => recursive = true,
                "--verbose"   => verbose = true,
                "--changes"   => changes = true,
                a if a.starts_with('-') => { eprintln!("chmod: unrecognized option '{}'", a); return 1; }
                _ => {
                    if mode_str.is_none() {
                        mode_str = Some(arg.clone());
                    } else {
                        paths.push(arg.clone());
                    }
                }
            }
        }
    }

    let mode_str = match mode_str {
        Some(m) => m,
        None => { eprintln!("chmod: missing operand"); return 1; }
    };

    if paths.is_empty() {
        eprintln!("chmod: missing operand after '{}'", mode_str);
        return 1;
    }

    let mut exit_code = 0;
    for path in &paths {
        exit_code |= chmod_path(Path::new(path), &mode_str, recursive, verbose, changes);
    }
    exit_code
}

fn chmod_path(path: &Path, mode_str: &str, recursive: bool, verbose: bool, changes: bool) -> i32 {
    let meta = match fs::metadata(path) {
        Ok(m) => m,
        Err(e) => { eprintln!("chmod: cannot access '{}': {}", path.display(), e); return 1; }
    };

    let old_mode = meta.permissions().mode();
    let new_mode = match apply_mode(old_mode, mode_str) {
        Some(m) => m,
        None => { eprintln!("chmod: invalid mode: '{}'", mode_str); return 1; }
    };

    // For directories under -R: recurse first, then apply (so we don't lose execute access)
    if recursive && meta.is_dir() {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                chmod_path(&entry.path(), mode_str, recursive, verbose, changes);
            }
        }
    }

    if new_mode != old_mode {
        let perms = fs::Permissions::from_mode(new_mode);
        if let Err(e) = fs::set_permissions(path, perms) {
            eprintln!("chmod: cannot change permissions of '{}': {}", path.display(), e);
            return 1;
        }
        if verbose || changes {
            println!("mode of '{}' changed from {:04o} to {:04o}", path.display(), old_mode & 0o7777, new_mode & 0o7777);
        }
    } else if verbose {
        println!("mode of '{}' retained as {:04o}", path.display(), old_mode & 0o7777);
    }

    0
}

fn apply_mode(current: u32, mode_str: &str) -> Option<u32> {
    // Try octal first
    if mode_str.chars().all(|c| c.is_ascii_digit()) {
        return u32::from_str_radix(mode_str, 8).ok();
    }

    // Symbolic mode: [ugoa]*[+-=][rwxXstugoa]*,...
    let mut mode = current & 0o7777;
    for clause in mode_str.split(',') {
        mode = apply_clause(mode, current, clause)?;
    }
    Some((current & !0o7777) | mode)
}

fn apply_clause(mode: u32, umask_ref: u32, clause: &str) -> Option<u32> {
    let mut chars = clause.chars().peekable();

    // Who: u g o a (default = a but respects umask, we use a)
    let mut who_u = false;
    let mut who_g = false;
    let mut who_o = false;
    let mut has_who = false;

    while let Some(&ch) = chars.peek() {
        match ch {
            'u' => { who_u = true; has_who = true; chars.next(); }
            'g' => { who_g = true; has_who = true; chars.next(); }
            'o' => { who_o = true; has_who = true; chars.next(); }
            'a' => { who_u = true; who_g = true; who_o = true; has_who = true; chars.next(); }
            _ => break,
        }
    }
    if !has_who { who_u = true; who_g = true; who_o = true; }

    // Op: + - =
    let op = match chars.next()? {
        '+' => '+',
        '-' => '-',
        '=' => '=',
        _ => return None,
    };

    // Perms
    let mut perm_bits = 0u32;
    let mut special_bits = 0u32;
    for ch in chars {
        match ch {
            'r' => {
                if who_u { perm_bits |= 0o400; }
                if who_g { perm_bits |= 0o040; }
                if who_o { perm_bits |= 0o004; }
            }
            'w' => {
                if who_u { perm_bits |= 0o200; }
                if who_g { perm_bits |= 0o020; }
                if who_o { perm_bits |= 0o002; }
            }
            'x' => {
                if who_u { perm_bits |= 0o100; }
                if who_g { perm_bits |= 0o010; }
                if who_o { perm_bits |= 0o001; }
            }
            'X' => {
                // Execute only if dir or already executable
                if umask_ref & 0o111 != 0 || umask_ref & 0o040000 != 0 {
                    if who_u { perm_bits |= 0o100; }
                    if who_g { perm_bits |= 0o010; }
                    if who_o { perm_bits |= 0o001; }
                }
            }
            's' => {
                if who_u { special_bits |= 0o4000; }
                if who_g { special_bits |= 0o2000; }
            }
            't' => { special_bits |= 0o1000; }
            'u' => {
                let u_bits = (umask_ref >> 6) & 0o7;
                if who_u { perm_bits |= u_bits << 6; }
                if who_g { perm_bits |= u_bits << 3; }
                if who_o { perm_bits |= u_bits; }
            }
            'g' => {
                let g_bits = (umask_ref >> 3) & 0o7;
                if who_u { perm_bits |= g_bits << 6; }
                if who_g { perm_bits |= g_bits << 3; }
                if who_o { perm_bits |= g_bits; }
            }
            'o' => {
                let o_bits = umask_ref & 0o7;
                if who_u { perm_bits |= o_bits << 6; }
                if who_g { perm_bits |= o_bits << 3; }
                if who_o { perm_bits |= o_bits; }
            }
            _ => return None,
        }
    }

    let all_bits = perm_bits | special_bits;

    // Build mask of bits we're touching
    let mut mask = 0u32;
    if who_u { mask |= 0o4700; }
    if who_g { mask |= 0o2070; }
    if who_o { mask |= 0o1007; }

    let new_mode = match op {
        '+' => mode | all_bits,
        '-' => mode & !all_bits,
        '=' => (mode & !mask) | all_bits,
        _   => return None,
    };

    Some(new_mode)
}
