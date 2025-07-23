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
