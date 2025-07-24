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

// Simple BRE-style regex match anchored at start.
// Supports: . * [] [^] \(\) ^ $
// Returns the capture group string if present, else the match length.
fn do_match(s: &str, pattern: &str) -> String {
    let anchored = if pattern.starts_with('^') { pattern.to_string() } else { format!("^{}", pattern) };
    let has_group = anchored.contains("\\(");
    let s_chars: Vec<char> = s.chars().collect();
    let pat_chars: Vec<char> = anchored.chars().collect();

    if let Some((end, cap)) = bre_match(&s_chars, 0, &pat_chars, 0) {
        if has_group {
            if let Some((gs, ge)) = cap { s_chars[gs..ge].iter().collect() } else { String::new() }
        } else {
            end.to_string()
        }
    } else {
        if has_group { String::new() } else { "0".to_string() }
    }
}

// Returns (match_end_index, Option<(group_start, group_end)>)
fn bre_match(s: &[char], si: usize, pat: &[char], pi: usize) -> Option<(usize, Option<(usize, usize)>)> {
    bre_rec(s, si, pat, pi, None, None)
}

fn bre_rec(
    s: &[char], si: usize,
    pat: &[char], pi: usize,
    grp_start: Option<usize>,
    grp_end: Option<usize>,
) -> Option<(usize, Option<(usize, usize)>)> {
    if pi >= pat.len() {
        return Some((si, grp_start.zip(grp_end).or_else(|| grp_start.map(|g| (g, si)))));
    }

    // Parse one atom from pat[pi..]
    // Returns (atom_end_in_pat, is_star)
    let (atom_end, atom_type) = parse_atom(pat, pi)?;
    let is_star = pat.get(atom_end) == Some(&'*') && atom_type != AtomType::GroupStart && atom_type != AtomType::GroupEnd;
    let next_pi = if is_star { atom_end + 1 } else { atom_end };

    match atom_type {
        AtomType::Anchor => bre_rec(s, si, pat, next_pi, grp_start, grp_end),
        AtomType::End => {
            if si == s.len() { bre_rec(s, si, pat, next_pi, grp_start, grp_end) } else { None }
        }
        AtomType::GroupStart => {
            bre_rec(s, si, pat, next_pi, Some(si), grp_end)
        }
        AtomType::GroupEnd => {
            bre_rec(s, si, pat, next_pi, grp_start, Some(si))
        }
        _ => {
            if is_star {
                // Greedy: try matching as many as possible, then backtrack
                let mut positions = vec![si];
                let mut cur = si;
                loop {
                    match match_one(&atom_type, s, cur) {
                        Some(next) => { positions.push(next); cur = next; }
                        None => break,
                    }
                }
                for &pos in positions.iter().rev() {
                    if let Some(r) = bre_rec(s, pos, pat, next_pi, grp_start, grp_end) {
                        return Some(r);
                    }
                }
                None
            } else {
                let next_si = match_one(&atom_type, s, si)?;
                bre_rec(s, next_si, pat, next_pi, grp_start, grp_end)
            }
        }
    }
}

#[derive(PartialEq)]
enum AtomType {
    Any,
    Literal(char),
    Class(Vec<char>, bool), // chars, negated
    GroupStart,
    GroupEnd,
    Anchor, // ^ at start
    End,    // $ at end
}

// Returns (pat index after atom, AtomType)
fn parse_atom(pat: &[char], pi: usize) -> Option<(usize, AtomType)> {
    match pat[pi] {
        '^' => Some((pi + 1, AtomType::Anchor)),
        '$' => Some((pi + 1, AtomType::End)),
        '.' => Some((pi + 1, AtomType::Any)),
        '*' => Some((pi + 1, AtomType::Literal('*'))), // unattached * is literal
        '\\' if pi + 1 < pat.len() => match pat[pi + 1] {
            '(' => Some((pi + 2, AtomType::GroupStart)),
            ')' => Some((pi + 2, AtomType::GroupEnd)),
            c   => Some((pi + 2, AtomType::Literal(c))),
        },
        '[' => {
            let mut i = pi + 1;
            let negated = i < pat.len() && pat[i] == '^';
            if negated { i += 1; }
            let mut chars = Vec::new();
            if i < pat.len() && pat[i] == ']' { chars.push(']'); i += 1; }
            while i < pat.len() && pat[i] != ']' {
                if i + 2 < pat.len() && pat[i + 1] == '-' && pat[i + 2] != ']' {
                    let lo = pat[i]; let hi = pat[i + 2];
                    for c in lo..=hi { chars.push(c); }
                    i += 3;
                } else {
                    chars.push(pat[i]);
                    i += 1;
                }
            }
            if i < pat.len() { i += 1; } // consume ']'
            Some((i, AtomType::Class(chars, negated)))
        }
        c => Some((pi + 1, AtomType::Literal(c))),
    }
}

fn match_one(atom: &AtomType, s: &[char], si: usize) -> Option<usize> {
    if si >= s.len() { return None; }
    let ok = match atom {
        AtomType::Any => true,
        AtomType::Literal(c) => s[si] == *c,
        AtomType::Class(chars, negated) => chars.contains(&s[si]) != *negated,
        _ => return None,
    };
    if ok { Some(si + 1) } else { None }
}
