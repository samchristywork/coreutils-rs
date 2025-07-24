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
//   add: mul ( ('+'|'-') mul )*
//   mul: match ( ('*'|'/'|'%') match )*
//   match: primary ( ':' primary )*
//   primary: '(' or ')' | function | token

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

fn parse_add<'a>(tok: &'a [&'a str]) -> Result<(String, &'a [&'a str]), String> {
    let (mut lhs, mut rest) = parse_mul(tok)?;
    while let Some(&op) = rest.first() {
        if op != "+" && op != "-" { break; }
        let (rhs, r) = parse_mul(&rest[1..])?;
        let (a, b) = (parse_int(&lhs)?, parse_int(&rhs)?);
        lhs = match op {
            "+" => a.checked_add(b).ok_or("integer overflow")?.to_string(),
            "-" => a.checked_sub(b).ok_or("integer overflow")?.to_string(),
            _   => unreachable!(),
        };
        rest = r;
    }
    Ok((lhs, rest))
}

fn parse_mul<'a>(tok: &'a [&'a str]) -> Result<(String, &'a [&'a str]), String> {
    let (mut lhs, mut rest) = parse_match(tok)?;
    while let Some(&op) = rest.first() {
        if op != "*" && op != "/" && op != "%" { break; }
        let (rhs, r) = parse_match(&rest[1..])?;
        let (a, b) = (parse_int(&lhs)?, parse_int(&rhs)?);
        if (op == "/" || op == "%") && b == 0 { return Err("division by zero".to_string()); }
        lhs = match op {
            "*" => a.checked_mul(b).ok_or("integer overflow")?.to_string(),
            "/" => (a / b).to_string(),
            "%" => (a % b).to_string(),
            _   => unreachable!(),
        };
        rest = r;
    }
    Ok((lhs, rest))
}

fn parse_match<'a>(tok: &'a [&'a str]) -> Result<(String, &'a [&'a str]), String> {
    let (mut lhs, mut rest) = parse_primary(tok)?;
    while rest.first() == Some(&":") {
        let (rhs, r) = parse_primary(&rest[1..])?;
        lhs = do_match(&lhs, &rhs);
        rest = r;
    }
    Ok((lhs, rest))
}

fn parse_primary<'a>(tok: &'a [&'a str]) -> Result<(String, &'a [&'a str]), String> {
    if tok.is_empty() { return Err("missing operand".to_string()); }
    if tok[0] == "(" {
        let (val, rest) = parse_or(&tok[1..])?;
        if rest.first() != Some(&")") { return Err("expected ')'".to_string()); }
        return Ok((val, &rest[1..]));
    }
    match tok[0] {
        "length" => {
            let (val, rest) = parse_primary(&tok[1..])?;
            Ok((val.len().to_string(), rest))
        }
        "index" => {
            let (s, rest) = parse_primary(&tok[1..])?;
            let (chars, rest) = parse_primary(rest)?;
            let idx = s.chars().enumerate()
                .find(|(_, c)| chars.contains(*c))
                .map(|(i, _)| i + 1)
                .unwrap_or(0);
            Ok((idx.to_string(), rest))
        }
        "substr" => {
            let (s, rest) = parse_primary(&tok[1..])?;
            let (pos_s, rest) = parse_primary(rest)?;
            let (len_s, rest) = parse_primary(rest)?;
            let pos: i64 = pos_s.parse().unwrap_or(0);
            let len: i64 = len_s.parse().unwrap_or(0);
            let result = if pos < 1 || len < 1 {
                String::new()
            } else {
                let v: Vec<char> = s.chars().collect();
                let start = (pos as usize - 1).min(v.len());
                let end = (start + len as usize).min(v.len());
                v[start..end].iter().collect()
            };
            Ok((result, rest))
        }
        "match" => {
            let (s, rest) = parse_primary(&tok[1..])?;
            let (pat, rest) = parse_primary(rest)?;
            Ok((do_match(&s, &pat), rest))
        }
        _ => Ok((tok[0].to_string(), &tok[1..])),
    }
}
