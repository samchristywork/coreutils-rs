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

pub fn sha1(r: &mut dyn Read) -> io::Result<String> {
    let data = read_all(r)?;
    Ok(sha1_digest(&data))
}

fn sha1_digest(input: &[u8]) -> String {
    let bit_len = (input.len() as u64).wrapping_mul(8);
    let mut msg = input.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 { msg.push(0); }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    let (mut h0, mut h1, mut h2, mut h3, mut h4) = (
        0x67452301u32, 0xEFCDAB89u32, 0x98BADCFEu32, 0x10325476u32, 0xC3D2E1F0u32,
    );

    for chunk in msg.chunks(64) {
        let mut w = [0u32; 80];
        for (i, word) in w[..16].iter_mut().enumerate() {
            *word = u32::from_be_bytes(chunk[i*4..i*4+4].try_into().unwrap());
        }
        for i in 16..80 {
            w[i] = (w[i-3] ^ w[i-8] ^ w[i-14] ^ w[i-16]).rotate_left(1);
        }
        let (mut a, mut b, mut c, mut d, mut e) = (h0, h1, h2, h3, h4);
        for (i, &wi) in w.iter().enumerate() {
            let (f, k) = match i {
                0..=19  => ((b & c) | (!b & d),              0x5A827999u32),
                20..=39 => (b ^ c ^ d,                       0x6ED9EBA1u32),
                40..=59 => ((b & c) | (b & d) | (c & d),    0x8F1BBCDCu32),
                _       => (b ^ c ^ d,                       0xCA62C1D6u32),
            };
            let temp = a.rotate_left(5).wrapping_add(f).wrapping_add(e).wrapping_add(k).wrapping_add(wi);
            e = d; d = c; c = b.rotate_left(30); b = a; a = temp;
        }
        h0 = h0.wrapping_add(a); h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c); h3 = h3.wrapping_add(d); h4 = h4.wrapping_add(e);
    }

    format!("{:08x}{:08x}{:08x}{:08x}{:08x}", h0, h1, h2, h3, h4)
}

pub fn sha256(r: &mut dyn Read) -> io::Result<String> {
    let data = read_all(r)?;
    Ok(sha256_digest(&data))
}

fn sha256_digest(input: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98,0x71374491,0xb5c0fbcf,0xe9b5dba5,0x3956c25b,0x59f111f1,0x923f82a4,0xab1c5ed5,
        0xd807aa98,0x12835b01,0x243185be,0x550c7dc3,0x72be5d74,0x80deb1fe,0x9bdc06a7,0xc19bf174,
        0xe49b69c1,0xefbe4786,0x0fc19dc6,0x240ca1cc,0x2de92c6f,0x4a7484aa,0x5cb0a9dc,0x76f988da,
        0x983e5152,0xa831c66d,0xb00327c8,0xbf597fc7,0xc6e00bf3,0xd5a79147,0x06ca6351,0x14292967,
        0x27b70a85,0x2e1b2138,0x4d2c6dfc,0x53380d13,0x650a7354,0x766a0abb,0x81c2c92e,0x92722c85,
        0xa2bfe8a1,0xa81a664b,0xc24b8b70,0xc76c51a3,0xd192e819,0xd6990624,0xf40e3585,0x106aa070,
        0x19a4c116,0x1e376c08,0x2748774c,0x34b0bcb5,0x391c0cb3,0x4ed8aa4a,0x5b9cca4f,0x682e6ff3,
        0x748f82ee,0x78a5636f,0x84c87814,0x8cc70208,0x90befffa,0xa4506ceb,0xbef9a3f7,0xc67178f2,
    ];
    let init: [u32; 8] = [
        0x6a09e667,0xbb67ae85,0x3c6ef372,0xa54ff53a,0x510e527f,0x9b05688c,0x1f83d9ab,0x5be0cd19,
    ];

    let bit_len = (input.len() as u64).wrapping_mul(8);
    let mut msg = input.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 { msg.push(0); }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    let mut h = init;

    for chunk in msg.chunks(64) {
        let mut w = [0u32; 64];
        for (i, word) in w[..16].iter_mut().enumerate() {
            *word = u32::from_be_bytes(chunk[i*4..i*4+4].try_into().unwrap());
        }
        for i in 16..64 {
            let s0 = w[i-15].rotate_right(7) ^ w[i-15].rotate_right(18) ^ (w[i-15] >> 3);
            let s1 = w[i-2].rotate_right(17) ^ w[i-2].rotate_right(19)  ^ (w[i-2] >> 10);
            w[i] = w[i-16].wrapping_add(s0).wrapping_add(w[i-7]).wrapping_add(s1);
        }
        let (mut a,mut b,mut c,mut d,mut e,mut f,mut g,mut hh) = (h[0],h[1],h[2],h[3],h[4],h[5],h[6],h[7]);
        for i in 0..64 {
            let s1  = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch  = (e & f) ^ (!e & g);
            let t1  = hh.wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
            let s0  = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2  = s0.wrapping_add(maj);
            hh=g; g=f; f=e; e=d.wrapping_add(t1); d=c; c=b; b=a; a=t1.wrapping_add(t2);
        }
        let add = [a,b,c,d,e,f,g,hh];
        for (hi, &ai) in h.iter_mut().zip(add.iter()) { *hi = hi.wrapping_add(ai); }
    }

    format!("{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}",
        h[0],h[1],h[2],h[3],h[4],h[5],h[6],h[7])
}

pub fn sha512(r: &mut dyn Read) -> io::Result<String> {
    let data = read_all(r)?;
    Ok(sha512_digest(&data))
}

fn sha512_digest(input: &[u8]) -> String {
    const K: [u64; 80] = [
        0x428a2f98d728ae22,0x7137449123ef65cd,0xb5c0fbcfec4d3b2f,0xe9b5dba58189dbbc,
        0x3956c25bf348b538,0x59f111f1b605d019,0x923f82a4af194f9b,0xab1c5ed5da6d8118,
        0xd807aa98a3030242,0x12835b0145706fbe,0x243185be4ee4b28c,0x550c7dc3d5ffb4e2,
        0x72be5d74f27b896f,0x80deb1fe3b1696b1,0x9bdc06a725c71235,0xc19bf174cf692694,
        0xe49b69c19ef14ad2,0xefbe4786384f25e3,0x0fc19dc68b8cd5b5,0x240ca1cc77ac9c65,
        0x2de92c6f592b0275,0x4a7484aa6ea6e483,0x5cb0a9dcbd41fbd4,0x76f988da831153b5,
        0x983e5152ee66dfab,0xa831c66d2db43210,0xb00327c898fb213f,0xbf597fc7beef0ee4,
        0xc6e00bf33da88fc2,0xd5a79147930aa725,0x06ca6351e003826f,0x142929670a0e6e70,
        0x27b70a8546d22ffc,0x2e1b21385c26c926,0x4d2c6dfc5ac42aed,0x53380d139d95b3df,
        0x650a73548baf63de,0x766a0abb3c77b2a8,0x81c2c92e47edaee6,0x92722c851482353b,
        0xa2bfe8a14cf10364,0xa81a664bbc423001,0xc24b8b70d0f89791,0xc76c51a30654be30,
        0xd192e819d6ef5218,0xd69906245565a910,0xf40e35855771202a,0x106aa07032bbd1b8,
        0x19a4c116b8d2d0c8,0x1e376c085141ab53,0x2748774cdf8eeb99,0x34b0bcb5e19b48a8,
        0x391c0cb3c5c95a63,0x4ed8aa4ae3418acb,0x5b9cca4f7763e373,0x682e6ff3d6b2b8a3,
        0x748f82ee5defb2fc,0x78a5636f43172f60,0x84c87814a1f0ab72,0x8cc702081a6439ec,
        0x90befffa23631e28,0xa4506cebde82bde9,0xbef9a3f7b2c67915,0xc67178f2e372532b,
        0xca273eceea26619c,0xd186b8c721c0c207,0xeada7dd6cde0eb1e,0xf57d4f7fee6ed178,
        0x06f067aa72176fba,0x0a637dc5a2c898a6,0x113f9804bef90dae,0x1b710b35131c471b,
        0x28db77f523047d84,0x32caab7b40c72493,0x3c9ebe0a15c9bebc,0x431d67c49c100d4c,
        0x4cc5d4becb3e42b6,0x597f299cfc657e2a,0x5fcb6fab3ad6faec,0x6c44198c4a475817,
    ];
    let init: [u64; 8] = [
        0x6a09e667f3bcc908,0xbb67ae8584caa73b,0x3c6ef372fe94f82b,0xa54ff53a5f1d36f1,
        0x510e527fade682d1,0x9b05688c2b3e6c1f,0x1f83d9abfb41bd6b,0x5be0cd19137e2179,
    ];

    let bit_len = (input.len() as u128).wrapping_mul(8);
    let mut msg = input.to_vec();
    msg.push(0x80);
    while msg.len() % 128 != 112 { msg.push(0); }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    let mut h = init;

    for chunk in msg.chunks(128) {
        let mut w = [0u64; 80];
        for (i, word) in w[..16].iter_mut().enumerate() {
            *word = u64::from_be_bytes(chunk[i*8..i*8+8].try_into().unwrap());
        }
        for i in 16..80 {
            let s0 = w[i-15].rotate_right(1)  ^ w[i-15].rotate_right(8)  ^ (w[i-15] >> 7);
            let s1 = w[i-2].rotate_right(19) ^ w[i-2].rotate_right(61) ^ (w[i-2] >> 6);
            w[i] = w[i-16].wrapping_add(s0).wrapping_add(w[i-7]).wrapping_add(s1);
        }
        let (mut a,mut b,mut c,mut d,mut e,mut f,mut g,mut hh) = (h[0],h[1],h[2],h[3],h[4],h[5],h[6],h[7]);
        for i in 0..80 {
            let s1  = e.rotate_right(14) ^ e.rotate_right(18) ^ e.rotate_right(41);
            let ch  = (e & f) ^ (!e & g);
            let t1  = hh.wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
            let s0  = a.rotate_right(28) ^ a.rotate_right(34) ^ a.rotate_right(39);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2  = s0.wrapping_add(maj);
            hh=g; g=f; f=e; e=d.wrapping_add(t1); d=c; c=b; b=a; a=t1.wrapping_add(t2);
        }
        let add = [a,b,c,d,e,f,g,hh];
        for (hi, &ai) in h.iter_mut().zip(add.iter()) { *hi = hi.wrapping_add(ai); }
    }

    format!("{:016x}{:016x}{:016x}{:016x}{:016x}{:016x}{:016x}{:016x}",
        h[0],h[1],h[2],h[3],h[4],h[5],h[6],h[7])
}

fn read_all(r: &mut dyn Read) -> io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    r.read_to_end(&mut buf)?;
    Ok(buf)
}
