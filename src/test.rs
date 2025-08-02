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
