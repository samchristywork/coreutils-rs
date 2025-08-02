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
