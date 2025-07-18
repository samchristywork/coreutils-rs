use std::ffi::CString;
use std::io::{self, Write};

pub fn run(args: &[String]) -> i32 {
    let mut human = false;
    let mut block_size: u64 = 1024;
    let mut inodes = false;
    let mut paths: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            "-h" | "--human-readable" => human = true,
            "-i" | "--inodes" => inodes = true,
            "-k" => block_size = 1024,
            "-m" => block_size = 1024 * 1024,
            "--block-size" => {
                i += 1;
                if i >= args.len() { eprintln!("df: option requires an argument -- 'block-size'"); return 1; }
                match args[i].parse() {
                    Ok(n) => block_size = n,
                    Err(_) => { eprintln!("df: invalid block size '{}'", args[i]); return 1; }
                }
            }
            _ if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") => {
                for ch in arg[1..].chars() {
                    match ch {
                        'h' => human = true,
                        'i' => inodes = true,
                        'k' => block_size = 1024,
                        'm' => block_size = 1024 * 1024,
                        _ => { eprintln!("df: invalid option -- '{}'", ch); return 1; }
                    }
                }
            }
            _ if arg.starts_with('-') => { eprintln!("df: unrecognized option '{}'", arg); return 1; }
            _ => paths.push(args[i].clone()),
        }
        i += 1;
    }

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    if inodes {
        let _ = writeln!(out, "{:<20} {:>12} {:>12} {:>12} {:>6}  {}",
            "Filesystem", "Inodes", "IUsed", "IFree", "IUse%", "Mounted on");
    } else {
        let unit: String = if human { "Size".to_string() } else { format!("{}-blocks", block_size / 512) };
        let _ = writeln!(out, "{:<20} {:>12} {:>12} {:>12} {:>6}  {}",
            "Filesystem", unit, "Used", "Available", "Use%", "Mounted on");
    }

    let targets = if paths.is_empty() {
        mounted_filesystems()
    } else {
        paths.iter().map(|p| (String::new(), p.clone())).collect()
    };

    let mut exit_code = 0;
    for (fsname, path) in &targets {
        match query_statvfs(path) {
            Some(stat) => {
                let fs_display = if fsname.is_empty() { stat.fsname.clone() } else { fsname.clone() };
                if inodes {
                    let total = stat.files;
                    let free = stat.ffree;
                    let used = total.saturating_sub(free);
                    let pct = if total > 0 { used * 100 / total } else { 0 };
                    let _ = writeln!(out, "{:<20} {:>12} {:>12} {:>12} {:>5}%  {}",
                        fs_display, total, used, free, pct, stat.mount);
                } else {
                    let bsize = stat.bsize.max(1);
                    let total_bytes = stat.blocks * bsize;
                    let free_bytes = stat.bfree * bsize;
                    let avail_bytes = stat.bavail * bsize;
                    let used_bytes = total_bytes.saturating_sub(free_bytes);
                    let pct = if total_bytes > 0 { used_bytes * 100 / total_bytes } else { 0 };

                    if human {
                        let _ = writeln!(out, "{:<20} {:>12} {:>12} {:>12} {:>5}%  {}",
                            fs_display,
                            human_size(total_bytes), human_size(used_bytes),
                            human_size(avail_bytes), pct, stat.mount);
                    } else {
                        let _ = writeln!(out, "{:<20} {:>12} {:>12} {:>12} {:>5}%  {}",
                            fs_display,
                            total_bytes / block_size, used_bytes / block_size,
                            avail_bytes / block_size, pct, stat.mount);
                    }
                }
            }
            None => {
                eprintln!("df: {}: No such file or directory", path);
                exit_code = 1;
            }
        }
    }
    exit_code
}

struct StatVfs {
    fsname: String,
    mount: String,
    bsize: u64,
    blocks: u64,
    bfree: u64,
    bavail: u64,
    files: u64,
    ffree: u64,
}

#[repr(C)]
struct CStatvfs {
    f_bsize: u64,
    f_frsize: u64,
    f_blocks: u64,
    f_bfree: u64,
    f_bavail: u64,
    f_files: u64,
    f_ffree: u64,
    f_favail: u64,
    f_fsid: u64,
    f_flag: u64,
    f_namemax: u64,
    _pad: [u8; 32],
}

extern "C" {
    fn statvfs(path: *const i8, buf: *mut CStatvfs) -> i32;
}

fn query_statvfs(path: &str) -> Option<StatVfs> {
    let path_c = CString::new(path).ok()?;
    let mut buf = std::mem::MaybeUninit::<CStatvfs>::uninit();
    let ret = unsafe { statvfs(path_c.as_ptr(), buf.as_mut_ptr()) };
    if ret != 0 { return None; }
    let s = unsafe { buf.assume_init() };
    let mount = find_mount(path).unwrap_or_else(|| path.to_string());
    let fsname = find_fsname(&mount).unwrap_or_else(|| "unknown".to_string());
    Some(StatVfs {
        fsname,
        mount,
        bsize: s.f_frsize.max(s.f_bsize),
        blocks: s.f_blocks,
        bfree: s.f_bfree,
        bavail: s.f_bavail,
        files: s.f_files,
        ffree: s.f_ffree,
    })
}
