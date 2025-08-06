use std::io::{self, Read, Write};
use std::fs::File;

pub fn run(args: &[String]) -> i32 {
    let mut sysv = false;
    let mut paths: Vec<String> = Vec::new();

    for arg in args {
        match arg.as_str() {
            "-s" | "--sysv" => sysv = true,
            "-r"            => sysv = false,
            _ if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") => {
                for ch in arg[1..].chars() {
                    match ch {
                        's' => sysv = true,
                        'r' => sysv = false,
                        _ => { eprintln!("sum: invalid option -- '{}'", ch); return 1; }
                    }
                }
            }
            _ if arg.starts_with('-') => { eprintln!("sum: unrecognized option '{}'", arg); return 1; }
            _ => paths.push(arg.clone()),
        }
    }

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let mut exit_code = 0;

    if paths.is_empty() {
        let (cksum, blocks) = compute(&mut io::stdin(), sysv);
        let _ = writeln!(out, "{:05} {:5}", cksum, blocks);
    } else {
        for path in &paths {
            let result = if path == "-" {
                compute(&mut io::stdin(), sysv)
            } else {
                match File::open(path) {
                    Ok(mut f) => compute(&mut f, sysv),
                    Err(e) => { eprintln!("sum: {}: {}", path, e); exit_code = 1; continue; }
                }
            };
            let (cksum, blocks) = result;
            if paths.len() == 1 {
                let _ = writeln!(out, "{:05} {:5}", cksum, blocks);
            } else {
                let _ = writeln!(out, "{:05} {:5} {}", cksum, blocks, path);
            }
        }
    }
    exit_code
}

fn compute(r: &mut dyn Read, sysv: bool) -> (u32, u64) {
    let mut buf = [0u8; 8192];
    let mut total_bytes: u64 = 0;

    if sysv {
        let mut s: u32 = 0;
        loop {
            let n = r.read(&mut buf).unwrap_or(0);
            if n == 0 { break; }
            for &b in &buf[..n] { s = s.wrapping_add(b as u32); }
            total_bytes += n as u64;
        }
        let r = (s & 0xffff).wrapping_add(s >> 16);
        let r = (r & 0xffff).wrapping_add(r >> 16);
        let blocks = total_bytes.div_ceil(512);
        (r & 0xffff, blocks)
    } else {
        // BSD: 16-bit rotate-right checksum, 1024-byte blocks
        let mut cksum: u32 = 0;
        loop {
            let n = r.read(&mut buf).unwrap_or(0);
            if n == 0 { break; }
            for &b in &buf[..n] {
                cksum = (cksum >> 1) | ((cksum & 1) << 15);
                cksum = (cksum + b as u32) & 0xffff;
            }
            total_bytes += n as u64;
        }
        let blocks = total_bytes.div_ceil(1024);
        (cksum, blocks)
    }
}
