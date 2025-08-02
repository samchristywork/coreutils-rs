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
