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

fn format_statvfs(fmt: &str, s: &StatVfs, path: &str) -> String {
    let mut out = String::new();
    let mut chars = fmt.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '%' { out.push(ch); continue; }
        match chars.next() {
            Some('n') => out.push_str(path),
            Some('b') => out.push_str(&s.f_blocks.to_string()),
            Some('f') => out.push_str(&s.f_bfree.to_string()),
            Some('a') => out.push_str(&s.f_bavail.to_string()),
            Some('s') => out.push_str(&s.f_bsize.to_string()),
            Some('S') => out.push_str(&s.f_frsize.to_string()),
            Some('c') => out.push_str(&s.f_files.to_string()),
            Some('d') => out.push_str(&s.f_ffree.to_string()),
            Some('l') => out.push_str(&s.f_namemax.to_string()),
            Some('%') => out.push('%'),
            Some(c) => { out.push('%'); out.push(c); }
            None => out.push('%'),
        }
    }
    out
}

fn file_type_str(mode: u32) -> &'static str {
    match mode & 0o170000 {
        0o040000 => "directory",
        0o120000 => "symbolic link",
        0o100000 => "regular file",
        0o060000 => "block special file",
        0o020000 => "character special file",
        0o010000 => "fifo",
        0o140000 => "socket",
        _ => "unknown",
    }
}

fn format_mode(mode: u32) -> String {
    let ft = match mode & 0o170000 {
        0o040000 => 'd', 0o120000 => 'l', 0o100000 => '-',
        0o060000 => 'b', 0o020000 => 'c', 0o010000 => 'p',
        0o140000 => 's', _ => '?',
    };
    let rwx = [(0o400,'r'),(0o200,'w'),(0o100,'x'),
               (0o040,'r'),(0o020,'w'),(0o010,'x'),
               (0o004,'r'),(0o002,'w'),(0o001,'x')];
    let mut s = String::with_capacity(10);
    s.push(ft);
    for &(bit, ch) in &rwx { s.push(if mode & bit != 0 { ch } else { '-' }); }
    s
}

fn format_time(secs: i64) -> String {
    if secs < 0 { return "1970-01-01 00:00:00.000000000 +0000".to_string(); }
    let s = secs as u64;
    let min = s / 60; let hr = min / 60; let days = hr / 24;
    let (y, mo, d) = days_to_ymd(days);
    const MONTHS: [&str; 12] = ["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"];
    let _ = MONTHS;
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}.000000000 +0000",
        y, mo, d, hr % 24, min % 60, s % 60)
}

fn days_to_ymd(days: u64) -> (u64, u64, u64) {
    let mut r = days as i64; let mut y = 1970i64;
    loop {
        let dy = if is_leap(y) { 366 } else { 365 };
        if r < dy { break; } r -= dy; y += 1;
    }
    let md: [i64; 12] = [31, if is_leap(y) { 29 } else { 28 }, 31,30,31,30,31,31,30,31,30,31];
    let mut mo = 1u64;
    for &m in &md { if r < m { break; } r -= m; mo += 1; }
    (y as u64, mo, r as u64 + 1)
}

fn is_leap(y: i64) -> bool { y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) }

fn uid_name(uid: u32) -> String {
    use std::ffi::CStr;
    #[repr(C)]
    struct Passwd { pw_name: *const i8, _rest: [u8; 64] }
    extern "C" { fn getpwuid(uid: u32) -> *const Passwd; }
    let pw = unsafe { getpwuid(uid) };
    if pw.is_null() { return uid.to_string(); }
    unsafe { CStr::from_ptr((*pw).pw_name).to_string_lossy().into_owned() }
}

fn gid_name(gid: u32) -> String {
    use std::ffi::CStr;
    #[repr(C)]
    struct Group { gr_name: *const i8, _rest: [u8; 64] }
    extern "C" { fn getgrgid(gid: u32) -> *const Group; }
    let gr = unsafe { getgrgid(gid) };
    if gr.is_null() { return gid.to_string(); }
    unsafe { CStr::from_ptr((*gr).gr_name).to_string_lossy().into_owned() }
}
