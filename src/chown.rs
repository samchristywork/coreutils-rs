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
