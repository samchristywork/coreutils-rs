use std::fs;
use std::os::unix::fs::MetadataExt;

// Invoked as both `test` and `[`
pub fn run(args: &[String]) -> i32 {
    run_bracket(args, false)
}

pub fn run_bracket(args: &[String], bracket: bool) -> i32 {
    let args: &[String] = if bracket {
        match args.last() {
            Some(s) if s == "]" => &args[..args.len() - 1],
            _ => { eprintln!("[: missing ']'"); return 2; }
        }
    } else {
        args
    };

    if args.is_empty() { return 1; }

    let tokens: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    match parse_expr(&tokens) {
        Ok((result, _)) => if result { 0 } else { 1 },
        Err(e) => { eprintln!("test: {}", e); 2 }
    }
}

type ParseResult<'a> = Result<(bool, &'a [&'a str]), String>;

fn parse_expr<'a>(tok: &'a [&'a str]) -> ParseResult<'a> {
    parse_or(tok)
}

fn parse_or<'a>(tok: &'a [&'a str]) -> ParseResult<'a> {
    let (mut val, mut rest) = parse_and(tok)?;
    while rest.first() == Some(&"-o") {
        let (rhs, r) = parse_and(&rest[1..])?;
        val = val || rhs;
        rest = r;
    }
    Ok((val, rest))
}

fn parse_and<'a>(tok: &'a [&'a str]) -> ParseResult<'a> {
    let (mut val, mut rest) = parse_not(tok)?;
    while rest.first() == Some(&"-a") {
        let (rhs, r) = parse_not(&rest[1..])?;
        val = val && rhs;
        rest = r;
    }
    Ok((val, rest))
}

fn parse_not<'a>(tok: &'a [&'a str]) -> ParseResult<'a> {
    if tok.first() == Some(&"!") {
        let (val, rest) = parse_not(&tok[1..])?;
        Ok((!val, rest))
    } else {
        parse_primary(tok)
    }
}

fn parse_primary<'a>(tok: &'a [&'a str]) -> ParseResult<'a> {
    if tok.is_empty() {
        return Ok((false, tok));
    }

    // Parenthesized expression
    if tok[0] == "(" {
        let (val, rest) = parse_expr(&tok[1..])?;
        if rest.first() != Some(&")") {
            return Err("expected ')'".to_string());
        }
        return Ok((val, &rest[1..]));
    }

    // Unary operators
    if tok[0].starts_with('-') && tok[0].len() == 2 {
        let op = tok[0];
        // Peek ahead: if next token looks like a binary op, treat as string test
        if tok.len() >= 3 && is_binary_op(tok[1]) {
            // fall through to binary test
        } else if let Some(arg) = tok.get(1) {
            let result = eval_unary(op, arg)?;
            return Ok((result, &tok[2..]));
        } else if op == "-n" || op == "-z" {
            // -n and -z with no argument
            return Ok((op == "-n", &tok[1..]));
        }
    }

    // Expect: arg [binop arg] or just arg (string non-empty test)
    let lhs = tok[0];
    if tok.len() >= 3 && is_binary_op(tok[1]) {
        let op = tok[1];
        let rhs = tok[2];
        let result = eval_binary(lhs, op, rhs)?;
        return Ok((result, &tok[3..]));
    }

    // Single argument: true if non-empty string
    Ok((!lhs.is_empty(), &tok[1..]))
}

fn is_binary_op(s: &str) -> bool {
    matches!(s, "=" | "==" | "!=" | "<" | ">" |
               "-eq" | "-ne" | "-lt" | "-le" | "-gt" | "-ge")
}

fn eval_unary(op: &str, arg: &str) -> Result<bool, String> {
    match op {
        "-z" => Ok(arg.is_empty()),
        "-n" => Ok(!arg.is_empty()),
        "-e" => Ok(fs::symlink_metadata(arg).is_ok()),
        "-f" => Ok(fs::metadata(arg).map(|m| m.is_file()).unwrap_or(false)),
        "-d" => Ok(fs::metadata(arg).map(|m| m.is_dir()).unwrap_or(false)),
        "-r" => Ok(access(arg, 4)),
        "-w" => Ok(access(arg, 2)),
        "-x" => Ok(access(arg, 1)),
        "-s" => Ok(fs::metadata(arg).map(|m| m.len() > 0).unwrap_or(false)),
        "-L" | "-h" => Ok(fs::symlink_metadata(arg).map(|m| m.file_type().is_symlink()).unwrap_or(false)),
        "-p" => Ok(fs::metadata(arg).map(|m| (m.mode() & 0o170000) == 0o010000).unwrap_or(false)),
        "-S" => Ok(fs::metadata(arg).map(|m| (m.mode() & 0o170000) == 0o140000).unwrap_or(false)),
        "-b" => Ok(fs::metadata(arg).map(|m| (m.mode() & 0o170000) == 0o060000).unwrap_or(false)),
        "-c" => Ok(fs::metadata(arg).map(|m| (m.mode() & 0o170000) == 0o020000).unwrap_or(false)),
        "-g" => Ok(fs::metadata(arg).map(|m| m.mode() & 0o2000 != 0).unwrap_or(false)),
        "-u" => Ok(fs::metadata(arg).map(|m| m.mode() & 0o4000 != 0).unwrap_or(false)),
        "-k" => Ok(fs::metadata(arg).map(|m| m.mode() & 0o1000 != 0).unwrap_or(false)),
        "-t" => {
            let fd: i32 = arg.parse().unwrap_or(-1);
            Ok(isatty(fd))
        }
        "-O" => {
            extern "C" { fn getuid() -> u32; }
            Ok(fs::metadata(arg).map(|m| m.uid() == unsafe { getuid() }).unwrap_or(false))
        }
        "-G" => {
            extern "C" { fn getgid() -> u32; }
            Ok(fs::metadata(arg).map(|m| m.gid() == unsafe { getgid() }).unwrap_or(false))
        }
        "-N" => {
            // True if file modified since last read
            Ok(fs::metadata(arg).map(|m| m.mtime() >= m.atime()).unwrap_or(false))
        }
        _ => Err(format!("unknown unary operator '{}'", op)),
    }
}

fn eval_binary(lhs: &str, op: &str, rhs: &str) -> Result<bool, String> {
    match op {
        "=" | "==" => Ok(lhs == rhs),
        "!="       => Ok(lhs != rhs),
        "<"        => Ok(lhs < rhs),
        ">"        => Ok(lhs > rhs),
        "-eq" => int_cmp(lhs, rhs, |a, b| a == b),
        "-ne" => int_cmp(lhs, rhs, |a, b| a != b),
        "-lt" => int_cmp(lhs, rhs, |a, b| a <  b),
        "-le" => int_cmp(lhs, rhs, |a, b| a <= b),
        "-gt" => int_cmp(lhs, rhs, |a, b| a >  b),
        "-ge" => int_cmp(lhs, rhs, |a, b| a >= b),
        "-ef" => {
            let m1 = fs::metadata(lhs);
            let m2 = fs::metadata(rhs);
            Ok(match (m1, m2) {
                (Ok(a), Ok(b)) => a.dev() == b.dev() && a.ino() == b.ino(),
                _ => false,
            })
        }
        "-nt" => {
            let t1 = fs::metadata(lhs).map(|m| m.mtime()).unwrap_or(i64::MIN);
            let t2 = fs::metadata(rhs).map(|m| m.mtime()).unwrap_or(i64::MIN);
            Ok(t1 > t2)
        }
        "-ot" => {
            let t1 = fs::metadata(lhs).map(|m| m.mtime()).unwrap_or(i64::MAX);
            let t2 = fs::metadata(rhs).map(|m| m.mtime()).unwrap_or(i64::MAX);
            Ok(t1 < t2)
        }
        _ => Err(format!("unknown binary operator '{}'", op)),
    }
}

fn int_cmp(a: &str, b: &str, f: impl Fn(i64, i64) -> bool) -> Result<bool, String> {
    let x: i64 = a.trim().parse().map_err(|_| format!("integer expression expected: '{}'", a))?;
    let y: i64 = b.trim().parse().map_err(|_| format!("integer expression expected: '{}'", b))?;
    Ok(f(x, y))
}
