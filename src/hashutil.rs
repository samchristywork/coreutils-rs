use std::io::{self, Read, BufRead, Write};
use std::fs::File;

// Common runner for md5sum / sha*sum style commands.
// compute_fn streams from a reader and returns a lowercase hex digest.
pub fn run(args: &[String], compute_fn: fn(&mut dyn Read) -> io::Result<String>) -> i32 {
    let mut check = false;
    let mut binary = false;
    let mut quiet = false;
    let mut status_only = false;
    let mut warn = false;
    let mut paths: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            "-c" | "--check"        => check = true,
            "-b" | "--binary"       => binary = true,
            "-t" | "--text"         => binary = false,
            "-q" | "--quiet"        => quiet = true,
            "--status"              => status_only = true,
            "-w" | "--warn"         => warn = true,
            _ if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") => {
                for ch in arg[1..].chars() {
                    match ch {
                        'c' => check = true,
                        'b' => binary = true,
                        't' => binary = false,
                        'q' => quiet = true,
                        'w' => warn = true,
                        _ => { eprintln!("invalid option -- '{}'", ch); return 1; }
                    }
                }
            }
            _ if arg.starts_with('-') => { eprintln!("unrecognized option '{}'", arg); return 1; }
            _ => paths.push(arg.to_string()),
        }
        i += 1;
    }

    if check {
        run_check(&paths, compute_fn, quiet, status_only, warn)
    } else {
        run_hash(&paths, compute_fn, binary)
    }
}

fn run_hash(paths: &[String], compute_fn: fn(&mut dyn Read) -> io::Result<String>, binary: bool) -> i32 {
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let mut exit_code = 0;

    let mode_char = if binary { '*' } else { ' ' };

    if paths.is_empty() {
        match compute_fn(&mut io::stdin()) {
            Ok(digest) => { let _ = writeln!(out, "{}  -", digest); }
            Err(e) => { eprintln!("-: {}", e); exit_code = 1; }
        }
    } else {
        for path in paths {
            let result = if path == "-" {
                compute_fn(&mut io::stdin())
            } else {
                File::open(path).and_then(|mut f| compute_fn(&mut f))
            };
            match result {
                Ok(digest) => { let _ = writeln!(out, "{} {}{}", digest, mode_char, path); }
                Err(e) => { eprintln!("{}: {}", path, e); exit_code = 1; }
            }
        }
    }
    exit_code
}

fn run_check(
    paths: &[String],
    compute_fn: fn(&mut dyn Read) -> io::Result<String>,
    quiet: bool,
    status_only: bool,
    warn: bool,
) -> i32 {
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let mut exit_code = 0;

    let check_sources: Vec<String> = if paths.is_empty() {
        vec!["-".to_string()]
    } else {
        paths.to_vec()
    };

    for check_file in &check_sources {
        let reader: Box<dyn BufRead> = if check_file == "-" {
            Box::new(io::BufReader::new(io::stdin()))
        } else {
            match File::open(check_file) {
                Ok(f) => Box::new(io::BufReader::new(f)),
                Err(e) => { eprintln!("{}: {}", check_file, e); exit_code = 1; continue; }
            }
        };

        let mut line_num = 0u64;
        for line in reader.lines() {
            line_num += 1;
            let line = match line {
                Ok(l) => l,
                Err(e) => { eprintln!("{}: {}", check_file, e); exit_code = 1; break; }
            };
            let line = line.trim_end();
            if line.is_empty() || line.starts_with('#') { continue; }

            // Parse: "<hex>  <file>" or "<hex> *<file>"
            let (expected, target) = match parse_check_line(line) {
                Some(p) => p,
                None => {
                    if warn {
                        eprintln!("{}: {}: improperly formatted checksum line", check_file, line_num);
                    }
                    exit_code = 1;
                    continue;
                }
            };

            let actual = if target == "-" {
                compute_fn(&mut io::stdin())
            } else {
                File::open(&target).and_then(|mut f| compute_fn(&mut f))
            };

            match actual {
                Ok(digest) => {
                    if digest.eq_ignore_ascii_case(&expected) {
                        if !quiet && !status_only { let _ = writeln!(out, "{}: OK", target); }
                    } else {
                        if !status_only { let _ = writeln!(out, "{}: FAILED", target); }
                        exit_code = 1;
                    }
                }
                Err(e) => {
                    if !status_only { eprintln!("{}: {}", target, e); }
                    exit_code = 1;
                }
            }
        }
    }
    exit_code
}

fn parse_check_line(line: &str) -> Option<(String, String)> {
    // Format: "<hash>  <file>" (two spaces, text) or "<hash> *<file>" (space+star, binary)
    // The hash is a hex string (any length)
    let space = line.find("  ").or_else(|| line.find(" *"))?;
    let hash = line[..space].trim().to_string();
    if hash.is_empty() || !hash.chars().all(|c| c.is_ascii_hexdigit()) { return None; }
    let rest = &line[space + 2..];
    let filename = rest.trim_start_matches('*').to_string();
    if filename.is_empty() { return None; }
    Some((hash, filename))
}

pub fn md5(r: &mut dyn Read) -> io::Result<String> {
    let data = read_all(r)?;
    Ok(md5_digest(&data))
}

fn md5_digest(input: &[u8]) -> String {
    const S: [u32; 64] = [
        7,12,17,22, 7,12,17,22, 7,12,17,22, 7,12,17,22,
        5, 9,14,20, 5, 9,14,20, 5, 9,14,20, 5, 9,14,20,
        4,11,16,23, 4,11,16,23, 4,11,16,23, 4,11,16,23,
        6,10,15,21, 6,10,15,21, 6,10,15,21, 6,10,15,21,
    ];
    const K: [u32; 64] = [
        0xd76aa478,0xe8c7b756,0x242070db,0xc1bdceee,0xf57c0faf,0x4787c62a,0xa8304613,0xfd469501,
        0x698098d8,0x8b44f7af,0xffff5bb1,0x895cd7be,0x6b901122,0xfd987193,0xa679438e,0x49b40821,
        0xf61e2562,0xc040b340,0x265e5a51,0xe9b6c7aa,0xd62f105d,0x02441453,0xd8a1e681,0xe7d3fbc8,
        0x21e1cde6,0xc33707d6,0xf4d50d87,0x455a14ed,0xa9e3e905,0xfcefa3f8,0x676f02d9,0x8d2a4c8a,
        0xfffa3942,0x8771f681,0x6d9d6122,0xfde5380c,0xa4beea44,0x4bdecfa9,0xf6bb4b60,0xbebfbc70,
        0x289b7ec6,0xeaa127fa,0xd4ef3085,0x04881d05,0xd9d4d039,0xe6db99e5,0x1fa27cf8,0xc4ac5665,
        0xf4292244,0x432aff97,0xab9423a7,0xfc93a039,0x655b59c3,0x8f0ccc92,0xffeff47d,0x85845dd1,
        0x6fa87e4f,0xfe2ce6e0,0xa3014314,0x4e0811a1,0xf7537e82,0xbd3af235,0x2ad7d2bb,0xeb86d391,
    ];

    let bit_len = (input.len() as u64).wrapping_mul(8);
    let mut msg = input.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 { msg.push(0); }
    msg.extend_from_slice(&bit_len.to_le_bytes());

    let (mut a0, mut b0, mut c0, mut d0) = (0x67452301u32, 0xefcdab89u32, 0x98badcfeu32, 0x10325476u32);

    for chunk in msg.chunks(64) {
        let mut m = [0u32; 16];
        for (i, w) in m.iter_mut().enumerate() {
            *w = u32::from_le_bytes(chunk[i*4..i*4+4].try_into().unwrap());
        }
        let (mut a, mut b, mut c, mut d) = (a0, b0, c0, d0);
        for i in 0u32..64 {
            let (f, g) = match i {
                0..=15  => ((b & c) | (!b & d),          i),
                16..=31 => ((d & b) | (!d & c),          (5*i+1) % 16),
                32..=47 => (b ^ c ^ d,                   (3*i+5) % 16),
                _       => (c ^ (b | !d),                (7*i)   % 16),
            };
            let temp = d;
            let new_b = b.wrapping_add(f.wrapping_add(a).wrapping_add(K[i as usize]).wrapping_add(m[g as usize]).rotate_left(S[i as usize]));
            a = temp; d = c; c = b; b = new_b;
        }
        a0 = a0.wrapping_add(a); b0 = b0.wrapping_add(b);
        c0 = c0.wrapping_add(c); d0 = d0.wrapping_add(d);
    }

    format!("{:08x}{:08x}{:08x}{:08x}",
        a0.swap_bytes(), b0.swap_bytes(), c0.swap_bytes(), d0.swap_bytes())
}
