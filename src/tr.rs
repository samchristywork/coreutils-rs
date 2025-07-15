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
