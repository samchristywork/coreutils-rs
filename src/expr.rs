pub fn run(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("expr: missing operand");
        return 2;
    }
    let tokens: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    match parse_or(&tokens) {
        Ok((val, rest)) => {
            if !rest.is_empty() {
                eprintln!("expr: syntax error: unexpected token '{}'", rest[0]);
                return 2;
            }
            println!("{}", val);
            if is_null(&val) { 1 } else { 0 }
        }
        Err(e) => { eprintln!("expr: {}", e); 2 }
    }
}

fn is_null(s: &str) -> bool { s == "0" || s.is_empty() }

fn parse_int(s: &str) -> Result<i64, String> {
    s.trim().parse::<i64>().map_err(|_| format!("non-integer argument '{}'", s))
}

// Grammar (lowest to highest precedence):
//   or:  and ( '|' and )*
//   and: cmp ( '&' cmp )*
//   cmp: add ( ('='|'!='|'<'|'<='|'>'|'>=') add )?

fn parse_or<'a>(tok: &'a [&'a str]) -> Result<(String, &'a [&'a str]), String> {
    let (mut lhs, mut rest) = parse_and(tok)?;
    while rest.first() == Some(&"|") {
        let (rhs, r) = parse_and(&rest[1..])?;
        lhs = if !is_null(&lhs) { lhs } else if !is_null(&rhs) { rhs } else { "0".to_string() };
        rest = r;
    }
    Ok((lhs, rest))
}

fn parse_and<'a>(tok: &'a [&'a str]) -> Result<(String, &'a [&'a str]), String> {
    let (mut lhs, mut rest) = parse_cmp(tok)?;
    while rest.first() == Some(&"&") {
        let (rhs, r) = parse_cmp(&rest[1..])?;
        lhs = if !is_null(&lhs) && !is_null(&rhs) { lhs } else { "0".to_string() };
        rest = r;
    }
    Ok((lhs, rest))
}

fn parse_cmp<'a>(tok: &'a [&'a str]) -> Result<(String, &'a [&'a str]), String> {
    let (lhs, rest) = parse_add(tok)?;
    let op = match rest.first() {
        Some(&o) if matches!(o, "=" | "!=" | "<" | "<=" | ">" | ">=") => o,
        _ => return Ok((lhs, rest)),
    };
    let (rhs, rest) = parse_add(&rest[1..])?;
    let b = cmp_vals(&lhs, op, &rhs);
    Ok((if b { "1" } else { "0" }.to_string(), rest))
}

fn cmp_vals(a: &str, op: &str, b: &str) -> bool {
    if let (Ok(x), Ok(y)) = (a.parse::<i64>(), b.parse::<i64>()) {
        match op { "=" => x==y, "!=" => x!=y, "<" => x<y, "<=" => x<=y, ">" => x>y, ">=" => x>=y, _ => false }
    } else {
        match op { "=" => a==b, "!=" => a!=b, "<" => a<b, "<=" => a<=b, ">" => a>b, ">=" => a>=b, _ => false }
    }
}
