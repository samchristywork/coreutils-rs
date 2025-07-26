use std::io::{self, Read, Write};
use std::fs::File;

pub fn run(args: &[String]) -> i32 {
    let mut paths: Vec<String> = Vec::new();

    for arg in args {
        if arg.starts_with('-') {
            eprintln!("cksum: unrecognized option '{}'", arg);
            return 1;
        }
        paths.push(arg.clone());
    }

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let mut exit_code = 0;

    if paths.is_empty() {
        let (crc, nbytes) = compute(&mut io::stdin());
        let _ = writeln!(out, "{} {}", crc, nbytes);
    } else {
        for path in &paths {
            let result = if path == "-" {
                compute(&mut io::stdin())
            } else {
                match File::open(path) {
                    Ok(mut f) => compute(&mut f),
                    Err(e) => { eprintln!("cksum: {}: {}", path, e); exit_code = 1; continue; }
                }
            };
            let (crc, nbytes) = result;
            let _ = writeln!(out, "{} {} {}", crc, nbytes, path);
        }
    }
    exit_code
}

// POSIX CRC-32: polynomial 0x04C11DB7, MSB-first, length appended, final complement.
fn compute(r: &mut dyn Read) -> (u32, u64) {
    static TABLE: std::sync::OnceLock<[u32; 256]> = std::sync::OnceLock::new();
    let table = TABLE.get_or_init(|| {
        let mut t = [0u32; 256];
        for (i, entry) in t.iter_mut().enumerate() {
            let mut crc = (i as u32) << 24;
            for _ in 0..8 {
                crc = if crc & 0x8000_0000 != 0 { (crc << 1) ^ 0x04C11DB7 } else { crc << 1 };
            }
            *entry = crc;
        }
        t
    });

    let mut crc: u32 = 0;
    let mut nbytes: u64 = 0;
    let mut buf = [0u8; 8192];

    loop {
        let n = r.read(&mut buf).unwrap_or(0);
        if n == 0 { break; }
        for &b in &buf[..n] {
            crc = (crc << 8) ^ table[((crc >> 24) as u8 ^ b) as usize];
        }
        nbytes += n as u64;
    }

    // Append length bytes (little-endian, only significant octets)
    let mut len = nbytes;
    while len > 0 {
        crc = (crc << 8) ^ table[((crc >> 24) as u8 ^ (len & 0xff) as u8) as usize];
        len >>= 8;
    }

    (!crc, nbytes)
}
