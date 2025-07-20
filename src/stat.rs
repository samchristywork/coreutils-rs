use std::ffi::CString;
use std::io::{self, Write};

pub fn run(args: &[String]) -> i32 {
    let mut dereference = false;
    let mut format: Option<String> = None;
    let mut filesystem = false;
    let mut paths: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            "-L" | "--dereference" => dereference = true,
            "-f" | "--file-system" => filesystem = true,
            "-c" | "--format" => {
                i += 1;
                if i >= args.len() { eprintln!("stat: option requires an argument -- 'c'"); return 1; }
                format = Some(args[i].clone());
            }
            _ if arg.starts_with("--format=") => {
                format = Some(arg["--format=".len()..].to_string());
            }
            _ if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") => {
                let mut chars = arg[1..].chars().peekable();
                while let Some(ch) = chars.next() {
                    match ch {
                        'L' => dereference = true,
                        'f' => filesystem = true,
                        'c' => {
                            let rest: String = chars.collect();
                            let val = if rest.is_empty() {
                                i += 1;
                                if i >= args.len() { eprintln!("stat: option requires an argument -- 'c'"); return 1; }
                                args[i].clone()
                            } else { rest };
                            format = Some(val);
                            break;
                        }
                        _ => { eprintln!("stat: invalid option -- '{}'", ch); return 1; }
                    }
                }
            }
            _ if arg.starts_with('-') => { eprintln!("stat: unrecognized option '{}'", arg); return 1; }
            _ => paths.push(args[i].clone()),
        }
        i += 1;
    }

    if paths.is_empty() { eprintln!("stat: missing operand"); return 1; }

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let mut exit_code = 0;

    for path in &paths {
        exit_code |= stat_path(path, dereference, filesystem, format.as_deref(), &mut out);
    }
    exit_code
}

#[repr(C)]
struct StatBuf {
    st_dev: u64,
    st_ino: u64,
    st_nlink: u64,
    st_mode: u32,
    st_uid: u32,
    st_gid: u32,
    _pad0: u32,
    st_rdev: u64,
    st_size: i64,
    st_blksize: i64,
    st_blocks: i64,
    st_atime: i64,
    st_atime_nsec: i64,
    st_mtime: i64,
    st_mtime_nsec: i64,
    st_ctime: i64,
    st_ctime_nsec: i64,
    _unused: [i64; 3],
}

extern "C" {
    fn stat(path: *const i8, buf: *mut StatBuf) -> i32;
    fn lstat(path: *const i8, buf: *mut StatBuf) -> i32;
}

fn stat_path<W: Write>(path: &str, dereference: bool, filesystem: bool, format: Option<&str>, out: &mut W) -> i32 {
    let path_c = match CString::new(path) {
        Ok(c) => c,
        Err(_) => { eprintln!("stat: invalid path: '{}'", path); return 1; }
    };

    let mut buf = std::mem::MaybeUninit::<StatBuf>::uninit();
    let ret = unsafe {
        if dereference { stat(path_c.as_ptr(), buf.as_mut_ptr()) }
        else           { lstat(path_c.as_ptr(), buf.as_mut_ptr()) }
    };
    if ret != 0 {
        eprintln!("stat: cannot stat '{}': {}", path, io::Error::last_os_error());
        return 1;
    }
    let s = unsafe { buf.assume_init() };

    if filesystem {
        return stat_fs(path, format, out);
    }

    if let Some(fmt) = format {
        let line = format_stat(fmt, &s, path);
        let _ = writeln!(out, "{}", line);
        return 0;
    }

    // Default output
    let file_type = file_type_str(s.st_mode);
    let mode_str = format_mode(s.st_mode);
    let atime = format_time(s.st_atime);
    let mtime = format_time(s.st_mtime);
    let ctime = format_time(s.st_ctime);

    let link_target = if s.st_mode & 0o170000 == 0o120000 {
        match std::fs::read_link(path) {
            Ok(t) => format!(" -> {}", t.display()),
            Err(_) => String::new(),
        }
    } else {
        String::new()
    };

    let _ = writeln!(out, "  File: {}{}", path, link_target);
    let _ = writeln!(out, "  Size: {:<15} Blocks: {:<10} IO Block: {:<6} {}",
        s.st_size, s.st_blocks, s.st_blksize, file_type);
    let _ = writeln!(out, "Device: {:x}h/{:}d   Inode: {:<10} Links: {}",
        s.st_dev, s.st_dev, s.st_ino, s.st_nlink);
    let _ = writeln!(out, "Access: ({:04o}/{})  Uid: ({:5}/{})   Gid: ({:5}/{})",
        s.st_mode & 0o7777, mode_str,
        s.st_uid, uid_name(s.st_uid),
        s.st_gid, gid_name(s.st_gid));
    let _ = writeln!(out, "Access: {}", atime);
    let _ = writeln!(out, "Modify: {}", mtime);
    let _ = writeln!(out, "Change: {}", ctime);
    let _ = writeln!(out, " Birth: -");

    0
}

#[repr(C)]
struct StatVfs {
    f_bsize: i64, f_frsize: i64,
    f_blocks: u64, f_bfree: u64, f_bavail: u64,
    f_files: u64, f_ffree: u64, f_favail: u64,
    f_fsid: u64, f_flag: i64, f_namemax: i64,
    _pad: [i64; 6],
}

extern "C" { fn statvfs(path: *const i8, buf: *mut StatVfs) -> i32; }

fn stat_fs<W: Write>(path: &str, format: Option<&str>, out: &mut W) -> i32 {

    let path_c = match CString::new(path) { Ok(c) => c, Err(_) => return 1 };
    let mut buf = std::mem::MaybeUninit::<StatVfs>::uninit();
    if unsafe { statvfs(path_c.as_ptr(), buf.as_mut_ptr()) } != 0 {
        eprintln!("stat: cannot stat '{}': {}", path, io::Error::last_os_error());
        return 1;
    }
    let s = unsafe { buf.assume_init() };

    if let Some(fmt) = format {
        let line = format_statvfs(fmt, &s, path);
        let _ = writeln!(out, "{}", line);
        return 0;
    }

    let _ = writeln!(out, "  File: \"{}\"", path);
    let _ = writeln!(out, "    ID: {:>16x} Namelen: {:<6} Type: linux",
        s.f_fsid, s.f_namemax);
    let _ = writeln!(out, " Block size: {:<10} Fundamental block size: {}",
        s.f_bsize, s.f_frsize);
    let _ = writeln!(out, " Blocks: Total: {:<10} Free: {:<10} Available: {}",
        s.f_blocks, s.f_bfree, s.f_bavail);
    let _ = writeln!(out, "  Inodes: Total: {:<10} Free: {}",
        s.f_files, s.f_ffree);
    0
}

fn format_stat(fmt: &str, s: &StatBuf, path: &str) -> String {
    let mut out = String::new();
    let mut chars = fmt.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '%' { out.push(ch); continue; }
        match chars.next() {
            Some('n') => out.push_str(path),
            Some('s') => out.push_str(&s.st_size.to_string()),
            Some('b') => out.push_str(&s.st_blocks.to_string()),
            Some('B') => out.push_str(&s.st_blksize.to_string()),
            Some('f') => out.push_str(&format!("{:x}", s.st_mode)),
            Some('a') => out.push_str(&format!("{:o}", s.st_mode & 0o7777)),
            Some('A') => out.push_str(&format_mode(s.st_mode)),
            Some('i') => out.push_str(&s.st_ino.to_string()),
            Some('h') => out.push_str(&s.st_nlink.to_string()),
            Some('u') => out.push_str(&s.st_uid.to_string()),
            Some('g') => out.push_str(&s.st_gid.to_string()),
            Some('U') => out.push_str(&uid_name(s.st_uid)),
            Some('G') => out.push_str(&gid_name(s.st_gid)),
            Some('d') => out.push_str(&s.st_dev.to_string()),
            Some('D') => out.push_str(&format!("{:x}", s.st_dev)),
            Some('r') => out.push_str(&s.st_rdev.to_string()),
            Some('R') => out.push_str(&format!("{:x}", s.st_rdev)),
            Some('X') => out.push_str(&s.st_atime.to_string()),
            Some('Y') => out.push_str(&s.st_mtime.to_string()),
            Some('Z') => out.push_str(&s.st_ctime.to_string()),
            Some('x') => out.push_str(&format_time(s.st_atime)),
            Some('y') => out.push_str(&format_time(s.st_mtime)),
            Some('z') => out.push_str(&format_time(s.st_ctime)),
            Some('F') => out.push_str(file_type_str(s.st_mode)),
            Some('%') => out.push('%'),
            Some(c) => { out.push('%'); out.push(c); }
            None => out.push('%'),
        }
    }
    out
}
