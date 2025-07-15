use std::io::{self, Read, Write};

pub fn run(args: &[String]) -> i32 {
    let mut delete = false;
    let mut squeeze = false;
    let mut complement = false;
    let mut operands: Vec<String> = Vec::new();

    for arg in args {
        if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") {
            for ch in arg[1..].chars() {
                match ch {
                    'd' => delete = true,
                    's' => squeeze = true,
                    'c' | 'C' => complement = true,
                    _ => {
                        eprintln!("tr: invalid option -- '{}'", ch);
                        return 1;
                    }
                }
            }
        } else if arg.starts_with("--") {
            match arg.as_str() {
                "--delete" => delete = true,
                "--squeeze-repeats" => squeeze = true,
                "--complement" => complement = true,
                _ => {
                    eprintln!("tr: unrecognized option '{}'", arg);
                    return 1;
                }
            }
        } else {
            operands.push(arg.clone());
        }
    }

    if operands.is_empty() {
        eprintln!("tr: missing operand");
        return 1;
    }

    let set1_chars = match expand_set(&operands[0]) {
        Some(s) => s,
        None => { eprintln!("tr: invalid set1"); return 1; }
    };

    if delete && !squeeze {
        // tr -d SET1
        let set1: Vec<bool> = make_bool_table(&set1_chars, complement);
        return translate_stdin(|b| if set1[b as usize] { Action::Delete } else { Action::Keep }, false);
    }

    if delete && squeeze {
        // tr -ds SET1 SET2: delete SET1, then squeeze SET2
        if operands.len() < 2 {
            eprintln!("tr: missing operand after '{}'", operands[0]);
            return 1;
        }
        let set2_chars = match expand_set(&operands[1]) {
            Some(s) => s,
            None => { eprintln!("tr: invalid set2"); return 1; }
        };
        let del_table: Vec<bool> = make_bool_table(&set1_chars, complement);
        let sq_table: Vec<bool> = make_bool_table(&set2_chars, false);
        return translate_stdin(|b| {
            if del_table[b as usize] { Action::Delete }
            else if sq_table[b as usize] { Action::Squeeze }
            else { Action::Keep }
        }, true);
    }

    if squeeze && operands.len() == 1 {
        // tr -s SET1: squeeze repeated chars in SET1
        let sq_table: Vec<bool> = make_bool_table(&set1_chars, complement);
        return translate_stdin(|b| {
            if sq_table[b as usize] { Action::Squeeze } else { Action::Keep }
        }, true);
    }

    // Translation (with optional squeeze)
    if operands.len() < 2 {
        eprintln!("tr: missing operand after '{}'", operands[0]);
        return 1;
    }
    let set2_chars = match expand_set(&operands[1]) {
        Some(s) => s,
        None => { eprintln!("tr: invalid set2"); return 1; }
    };

    let map = build_map(&set1_chars, &set2_chars, complement);
    let sq_table: Vec<bool> = if squeeze {
        make_bool_table(&set2_chars, false)
    } else {
        vec![false; 256]
    };

    translate_stdin(|b| {
        let out = map[b as usize];
        if out == 255 { return Action::Keep; } // sentinel: not in set1 when complement
        let mapped = if map[b as usize] == b { b } else { map[b as usize] };
        if squeeze && sq_table[mapped as usize] { Action::MapSqueeze(mapped) }
        else { Action::Map(mapped) }
    }, squeeze)
}

#[derive(Clone, Copy)]
enum Action {
    Keep,
    Delete,
    Squeeze,
    Map(u8),
    MapSqueeze(u8),
}

fn translate_stdin<F>(f: F, _squeeze: bool) -> i32
where
    F: Fn(u8) -> Action,
{
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut inp = stdin.lock();
    let mut out = io::BufWriter::new(stdout.lock());

    let mut buf = [0u8; 65536];
    let mut last: Option<u8> = None;

    loop {
        let n = match inp.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(_) => return 1,
        };

        let mut out_buf = Vec::with_capacity(n);
        for &b in &buf[..n] {
            match f(b) {
                Action::Keep => {
                    out_buf.push(b);
                    last = Some(b);
                }
                Action::Delete => {
                    last = None;
                }
                Action::Squeeze => {
                    if last != Some(b) {
                        out_buf.push(b);
                        last = Some(b);
                    }
                }
                Action::Map(m) => {
                    out_buf.push(m);
                    last = Some(m);
                }
                Action::MapSqueeze(m) => {
                    if last != Some(m) {
                        out_buf.push(m);
                        last = Some(m);
                    }
                }
            }
        }

        if out.write_all(&out_buf).is_err() {
            return 1;
        }
    }
    0
}

fn make_bool_table(chars: &[u8], complement: bool) -> Vec<bool> {
    let mut table = vec![false; 256];
    for &c in chars {
        table[c as usize] = true;
    }
    if complement {
        table.iter_mut().for_each(|b| *b = !*b);
    }
    table
}

fn build_map(set1: &[u8], set2: &[u8], complement: bool) -> Vec<u8> {
    // Identity map
    let mut map: Vec<u8> = (0u8..=255).collect();

    if complement {
        // Characters NOT in set1 get mapped; map them in order to set2
        let in_set1: Vec<bool> = make_bool_table(set1, false);
        let last2 = *set2.last().unwrap_or(&b' ');
        let mut s2_idx = 0usize;
        for i in 0usize..256 {
            if !in_set1[i] {
                map[i] = if s2_idx < set2.len() {
                    let v = set2[s2_idx];
                    s2_idx += 1;
                    v
                } else {
                    last2
                };
            }
        }
    } else {
        let last2 = *set2.last().unwrap_or(&b' ');
        for (idx, &c) in set1.iter().enumerate() {
            map[c as usize] = if idx < set2.len() { set2[idx] } else { last2 };
        }
    }

    map
}

fn expand_set(s: &str) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i += 1;
            if i >= bytes.len() { return None; }
            let ch = match bytes[i] {
                b'n' => b'\n',
                b't' => b'\t',
                b'r' => b'\r',
                b'a' => b'\x07',
                b'b' => b'\x08',
                b'f' => b'\x0C',
                b'v' => b'\x0B',
                b'\\' => b'\\',
                b'0'..=b'7' => {
                    // Octal escape up to 3 digits
                    let mut val = (bytes[i] - b'0') as u32;
                    for _ in 0..2 {
                        if i + 1 < bytes.len() && bytes[i+1] >= b'0' && bytes[i+1] <= b'7' {
                            i += 1;
                            val = val * 8 + (bytes[i] - b'0') as u32;
                        } else {
                            break;
                        }
                    }
                    if val > 255 { return None; }
                    val as u8
                }
                c => c,
            };
            out.push(ch);
            i += 1;
        } else if i + 2 < bytes.len() && bytes[i + 1] == b'-' {
            // Range a-z
            let start = bytes[i];
            let end = bytes[i + 2];
            if end < start { return None; }
            for c in start..=end {
                out.push(c);
            }
            i += 3;
        } else if bytes[i] == b'[' {
            // POSIX classes [:alpha:] etc or [=c=] or [x*n] repetition
            if i + 1 < bytes.len() && bytes[i + 1] == b':' {
                let close = bytes[i..].windows(2).position(|w| w == b":]");
                if let Some(rel) = close {
                    let class = &s[i+2..i+rel];
                    expand_class(class, &mut out);
                    i += rel + 2;
                    continue;
                }
            }
            if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                // [=c=] equivalence class — just use the char
                if i + 4 < bytes.len() && bytes[i + 3] == b'=' && bytes[i + 4] == b']' {
                    out.push(bytes[i + 2]);
                    i += 5;
                    continue;
                }
            }
            if let Some(star_pos) = bytes[i+1..].iter().position(|&b| b == b'*') {
                let star_pos = star_pos + i + 1;
                if star_pos + 1 < bytes.len() && bytes[star_pos + 1] != b']' {
                    // [x*n] repeat x n times
                    let ch = bytes[i + 1];
                    let count_str = std::str::from_utf8(&bytes[star_pos+1..]).ok()?;
                    let end = count_str.find(']').unwrap_or(count_str.len());
                    let count: usize = count_str[..end].parse().unwrap_or(0);
                    if count == 0 {
                        // [x*] means fill to length of set1 — we can't know here, push one
                        out.push(ch);
                    } else {
                        for _ in 0..count { out.push(ch); }
                    }
                    i = star_pos + 1 + end + 1;
                    continue;
                }
            }
            out.push(bytes[i]);
            i += 1;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }

    Some(out)
}
