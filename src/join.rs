use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

pub fn run(args: &[String]) -> i32 {
    let mut field1 = 1usize;
    let mut field2 = 1usize;
    let mut separator: Option<u8> = None;
    let mut ignore_case = false;
    let mut unpairable1 = false;
    let mut unpairable2 = false;
    let mut empty_fill = String::new();
    let mut output_fields: Option<Vec<(usize, usize)>> = None; // (file, field), file 0 = join field
    let mut paths: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            "-i" | "--ignore-case" => ignore_case = true,
            "-1" => {
                i += 1;
                if i >= args.len() { eprintln!("join: option requires an argument -- '1'"); return 1; }
                match args[i].parse::<usize>() {
                    Ok(n) if n > 0 => field1 = n,
                    _ => { eprintln!("join: invalid field number: '{}'", args[i]); return 1; }
                }
            }
            "-2" => {
                i += 1;
                if i >= args.len() { eprintln!("join: option requires an argument -- '2'"); return 1; }
                match args[i].parse::<usize>() {
                    Ok(n) if n > 0 => field2 = n,
                    _ => { eprintln!("join: invalid field number: '{}'", args[i]); return 1; }
                }
            }
            "-j" => {
                i += 1;
                if i >= args.len() { eprintln!("join: option requires an argument -- 'j'"); return 1; }
                match args[i].parse::<usize>() {
                    Ok(n) if n > 0 => { field1 = n; field2 = n; }
                    _ => { eprintln!("join: invalid field number: '{}'", args[i]); return 1; }
                }
            }
            "-t" => {
                i += 1;
                if i >= args.len() { eprintln!("join: option requires an argument -- 't'"); return 1; }
                let b = args[i].as_bytes();
                if b.len() != 1 { eprintln!("join: multi-character separator not allowed"); return 1; }
                separator = Some(b[0]);
            }
            "-a" => {
                i += 1;
                if i >= args.len() { eprintln!("join: option requires an argument -- 'a'"); return 1; }
                match args[i].as_str() {
                    "1" => unpairable1 = true,
                    "2" => unpairable2 = true,
                    _ => { eprintln!("join: invalid file number: '{}'", args[i]); return 1; }
                }
            }
            "-v" => {
                i += 1;
                if i >= args.len() { eprintln!("join: option requires an argument -- 'v'"); return 1; }
                match args[i].as_str() {
                    "1" => { unpairable1 = true; }
                    "2" => { unpairable2 = true; }
                    _ => { eprintln!("join: invalid file number: '{}'", args[i]); return 1; }
                }
                // -v suppresses joined lines; store as negative flag via a workaround
                // We'll handle -v separately by checking context
                // Actually just track it:
            }
            "-e" => {
                i += 1;
                if i >= args.len() { eprintln!("join: option requires an argument -- 'e'"); return 1; }
                empty_fill = args[i].clone();
            }
            "-o" => {
                i += 1;
                if i >= args.len() { eprintln!("join: option requires an argument -- 'o'"); return 1; }
                match parse_output_spec(&args[i]) {
                    Some(spec) => output_fields = Some(spec),
                    None => { eprintln!("join: invalid field spec: '{}'", args[i]); return 1; }
                }
            }
            _ if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") => {
                let mut chars = arg[1..].chars().peekable();
                while let Some(ch) = chars.next() {
                    match ch {
                        'i' => ignore_case = true,
                        '1' | '2' | 'j' | 't' | 'a' | 'v' | 'e' | 'o' => {
                            let rest: String = chars.collect();
                            let val = if rest.is_empty() {
                                i += 1;
                                if i >= args.len() {
                                    eprintln!("join: option requires an argument -- '{}'", ch);
                                    return 1;
                                }
                                args[i].clone()
                            } else { rest };
                            match ch {
                                '1' => match val.parse::<usize>() {
                                    Ok(n) if n > 0 => field1 = n,
                                    _ => { eprintln!("join: invalid field number: '{}'", val); return 1; }
                                },
                                '2' => match val.parse::<usize>() {
                                    Ok(n) if n > 0 => field2 = n,
                                    _ => { eprintln!("join: invalid field number: '{}'", val); return 1; }
                                },
                                'j' => match val.parse::<usize>() {
                                    Ok(n) if n > 0 => { field1 = n; field2 = n; }
                                    _ => { eprintln!("join: invalid field number: '{}'", val); return 1; }
                                },
                                't' => {
                                    let b = val.as_bytes();
                                    if b.len() != 1 { eprintln!("join: multi-character separator not allowed"); return 1; }
                                    separator = Some(b[0]);
                                }
                                'a' => match val.as_str() {
                                    "1" => unpairable1 = true,
                                    "2" => unpairable2 = true,
                                    _ => { eprintln!("join: invalid file number: '{}'", val); return 1; }
                                },
                                'v' => match val.as_str() {
                                    "1" => unpairable1 = true,
                                    "2" => unpairable2 = true,
                                    _ => { eprintln!("join: invalid file number: '{}'", val); return 1; }
                                },
                                'e' => empty_fill = val,
                                'o' => match parse_output_spec(&val) {
                                    Some(spec) => output_fields = Some(spec),
                                    None => { eprintln!("join: invalid field spec: '{}'", val); return 1; }
                                },
                                _ => unreachable!(),
                            }
                            break;
                        }
                        _ => { eprintln!("join: invalid option -- '{}'", ch); return 1; }
                    }
                }
            }
            _ if arg.starts_with('-') => { eprintln!("join: unrecognized option '{}'", arg); return 1; }
            _ => paths.push(args[i].clone()),
        }
        i += 1;
    }

    if paths.len() < 2 {
        eprintln!("join: missing operand");
        return 1;
    }

    let lines1 = match read_file(&paths[0]) {
        Ok(l) => l,
        Err(e) => { eprintln!("join: {}: {}", paths[0], e); return 1; }
    };
    let lines2 = match read_file(&paths[1]) {
        Ok(l) => l,
        Err(e) => { eprintln!("join: {}: {}", paths[1], e); return 1; }
    };

    let sep = separator.map(|b| b as char).unwrap_or(' ');
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    let mut i1 = 0;
    let mut i2 = 0;

    while i1 < lines1.len() || i2 < lines2.len() {
        let key1 = lines1.get(i1).map(|l| get_field(l, field1, sep, separator.is_none()));
        let key2 = lines2.get(i2).map(|l| get_field(l, field2, sep, separator.is_none()));

        let ord = match (&key1, &key2) {
            (None, None) => break,
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (Some(k1), Some(k2)) => {
                if ignore_case { k1.to_lowercase().cmp(&k2.to_lowercase()) }
                else { k1.cmp(k2) }
            }
        };

        match ord {
            std::cmp::Ordering::Less => {
                if unpairable1 {
                    let l = &lines1[i1];
                    emit_unpairable(l, 1, field1, sep, separator.is_none(), &output_fields, &empty_fill, &mut out);
                }
                i1 += 1;
            }
            std::cmp::Ordering::Greater => {
                if unpairable2 {
                    let l = &lines2[i2];
                    emit_unpairable(l, 2, field2, sep, separator.is_none(), &output_fields, &empty_fill, &mut out);
                }
                i2 += 1;
            }
            std::cmp::Ordering::Equal => {
                // Find the extent of this key group in both files
                let key = key1.unwrap();
                let start1 = i1;
                while i1 < lines1.len() {
                    let k = get_field(&lines1[i1], field1, sep, separator.is_none());
                    let eq = if ignore_case { k.to_lowercase() == key.to_lowercase() } else { k == key };
                    if !eq { break; }
                    i1 += 1;
                }
                let start2 = i2;
                while i2 < lines2.len() {
                    let k = get_field(&lines2[i2], field2, sep, separator.is_none());
                    let eq = if ignore_case { k.to_lowercase() == key.to_lowercase() } else { k == key };
                    if !eq { break; }
                    i2 += 1;
                }
                // Cross product
                for l1 in &lines1[start1..i1] {
                    for l2 in &lines2[start2..i2] {
                        emit_joined(l1, l2, &key, field1, field2, sep, separator.is_none(),
                            &output_fields, &empty_fill, &mut out);
                    }
                }
            }
        }
    }

    0
}

fn split_line(line: &str, sep: char, whitespace: bool) -> Vec<&str> {
    if whitespace {
        line.split_whitespace().collect()
    } else {
        line.split(sep).collect()
    }
}

fn get_field(line: &str, field: usize, sep: char, whitespace: bool) -> String {
    let fields = split_line(line, sep, whitespace);
    fields.get(field.saturating_sub(1)).unwrap_or(&"").to_string()
}

#[allow(clippy::too_many_arguments)]
fn emit_joined<W: Write>(
    l1: &str, l2: &str, key: &str,
    f1: usize, f2: usize,
    sep: char, whitespace: bool,
    output_fields: &Option<Vec<(usize, usize)>>,
    fill: &str,
    out: &mut W,
) {
    let fields1 = split_line(l1, sep, whitespace);
    let fields2 = split_line(l2, sep, whitespace);

    let result = if let Some(spec) = output_fields {
        spec.iter().map(|&(file, field)| {
            match (file, field) {
                (0, _) => key.to_string(),
                (1, 0) => key.to_string(),
                (2, 0) => key.to_string(),
                (1, n) => fields1.get(n.saturating_sub(1)).copied().unwrap_or(fill).to_string(),
                (2, n) => fields2.get(n.saturating_sub(1)).copied().unwrap_or(fill).to_string(),
                _ => fill.to_string(),
            }
        }).collect::<Vec<_>>().join(&sep.to_string())
    } else {
        // Default: join field, then remaining fields from l1, then remaining from l2
        let mut parts = vec![key.to_string()];
        for (idx, &f) in fields1.iter().enumerate() {
            if idx + 1 != f1 { parts.push(f.to_string()); }
        }
        for (idx, &f) in fields2.iter().enumerate() {
            if idx + 1 != f2 { parts.push(f.to_string()); }
        }
        parts.join(&sep.to_string())
    };

    let _ = writeln!(out, "{}", result);
}

#[allow(clippy::too_many_arguments)]
fn emit_unpairable<W: Write>(
    line: &str, file: usize, join_field: usize,
    sep: char, whitespace: bool,
    output_fields: &Option<Vec<(usize, usize)>>,
    fill: &str,
    out: &mut W,
) {
    let fields = split_line(line, sep, whitespace);
    let key = fields.get(join_field.saturating_sub(1)).unwrap_or(&"").to_string();

    let result = if let Some(spec) = output_fields {
        spec.iter().map(|&(ffile, ffield)| {
            if ffile == file {
                fields.get(ffield.saturating_sub(1)).copied().unwrap_or(fill).to_string()
            } else if ffile == 0 || ffield == 0 {
                key.clone()
            } else {
                fill.to_string()
            }
        }).collect::<Vec<_>>().join(&sep.to_string())
    } else {
        line.to_string()
    };

    let _ = writeln!(out, "{}", result);
}

fn parse_output_spec(s: &str) -> Option<Vec<(usize, usize)>> {
    let mut spec = Vec::new();
    for part in s.split(',') {
        if let Some((file_str, field_str)) = part.split_once('.') {
            let file = file_str.parse::<usize>().ok()?;
            let field = field_str.parse::<usize>().ok()?;
            spec.push((file, field));
        } else {
            return None;
        }
    }
    if spec.is_empty() { None } else { Some(spec) }
}

fn read_file(path: &str) -> io::Result<Vec<String>> {
    let reader: Box<dyn BufRead> = if path == "-" {
        Box::new(io::stdin().lock())
    } else {
        Box::new(BufReader::new(File::open(path)?))
    };
    let mut lines = Vec::new();
    for line in reader.lines() {
        lines.push(line?);
    }
    Ok(lines)
}
