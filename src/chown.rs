use std::ffi::CString;
use std::path::Path;
use std::fs;

pub fn run(args: &[String]) -> i32 {
    run_impl(args, false)
}

pub fn run_chgrp(args: &[String]) -> i32 {
    run_impl(args, true)
}

fn run_impl(args: &[String], group_only: bool) -> i32 {
    let mut recursive = false;
    let mut verbose = false;
    let mut changes = false;
    let mut owner_str: Option<String> = None;
    let mut paths: Vec<String> = Vec::new();

    for arg in args {
        if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") {
            for ch in arg[1..].chars() {
                match ch {
                    'R' => recursive = true,
                    'v' => verbose = true,
                    'c' => changes = true,
                    _ => {
                        let cmd = if group_only { "chgrp" } else { "chown" };
                        eprintln!("{}: invalid option -- '{}'", cmd, ch);
                        return 1;
                    }
                }
            }
        } else {
            match arg.as_str() {
                "--recursive" => recursive = true,
                "--verbose"   => verbose = true,
                "--changes"   => changes = true,
                a if a.starts_with('-') => {
                    let cmd = if group_only { "chgrp" } else { "chown" };
                    eprintln!("{}: unrecognized option '{}'", cmd, a);
                    return 1;
                }
                _ => {
                    if owner_str.is_none() {
                        owner_str = Some(arg.clone());
                    } else {
                        paths.push(arg.clone());
                    }
                }
            }
        }
    }

    let spec = match owner_str {
        Some(s) => s,
        None => {
            let cmd = if group_only { "chgrp" } else { "chown" };
            eprintln!("{}: missing operand", cmd);
            return 1;
        }
    };

    if paths.is_empty() {
        let cmd = if group_only { "chgrp" } else { "chown" };
        eprintln!("{}: missing operand after '{}'", cmd, spec);
        return 1;
    }

    let (uid, gid) = match parse_owner(&spec, group_only) {
        Some(p) => p,
        None => {
            let cmd = if group_only { "chgrp" } else { "chown" };
            eprintln!("{}: invalid spec: '{}'", cmd, spec);
            return 1;
        }
    };

    let mut exit_code = 0;
    for path in &paths {
        exit_code |= chown_path(Path::new(path), uid, gid, recursive, verbose, changes, group_only);
    }
    exit_code
}

fn chown_path(path: &Path, uid: Option<u32>, gid: Option<u32>, recursive: bool, verbose: bool, changes: bool, group_only: bool) -> i32 {
    let cmd = if group_only { "chgrp" } else { "chown" };

    let meta = match fs::metadata(path) {
        Ok(m) => m,
        Err(e) => { eprintln!("{}: cannot access '{}': {}", cmd, path.display(), e); return 1; }
    };

    use std::os::unix::fs::MetadataExt;
    let old_uid = meta.uid();
    let old_gid = meta.gid();
    let new_uid = uid.unwrap_or(old_uid);
    let new_gid = gid.unwrap_or(old_gid);

    let path_c = match CString::new(path.to_string_lossy().as_bytes()) {
        Ok(c) => c,
        Err(_) => { eprintln!("{}: invalid path", cmd); return 1; }
    };

    extern "C" {
        fn lchown(path: *const i8, owner: u32, group: u32) -> i32;
    }

    let ret = unsafe { lchown(path_c.as_ptr(), new_uid, new_gid) };
    if ret != 0 {
        let err = std::io::Error::last_os_error();
        eprintln!("{}: changing ownership of '{}': {}", cmd, path.display(), err);
        return 1;
    }

    if verbose || (changes && (new_uid != old_uid || new_gid != old_gid)) {
        println!("ownership of '{}' changed to {}:{}", path.display(), new_uid, new_gid);
    }

    if recursive && meta.is_dir() {
        if let Ok(entries) = fs::read_dir(path) {
            let mut code = 0;
            for entry in entries.flatten() {
                code |= chown_path(&entry.path(), uid, gid, recursive, verbose, changes, group_only);
            }
            return code;
        }
    }

    0
}

fn parse_owner(spec: &str, group_only: bool) -> Option<(Option<u32>, Option<u32>)> {
    if group_only {
        let gid = resolve_group(spec)?;
        return Some((None, Some(gid)));
    }

    if let Some((user_part, group_part)) = spec.split_once(':').or_else(|| spec.split_once('.')) {
        let uid = if user_part.is_empty() { None } else { Some(resolve_user(user_part)?) };
        let gid = if group_part.is_empty() { None } else { Some(resolve_group(group_part)?) };
        Some((uid, gid))
    } else {
        let uid = resolve_user(spec)?;
        Some((Some(uid), None))
    }
}
