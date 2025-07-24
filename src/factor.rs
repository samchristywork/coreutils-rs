use std::io::{self, BufRead, Write};

pub fn run(args: &[String]) -> i32 {
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let mut exit_code = 0;

    if args.is_empty() {
        // Read from stdin
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            match line {
                Ok(l) => {
                    for tok in l.split_whitespace() {
                        exit_code |= factor_one(tok, &mut out);
                    }
                }
                Err(e) => { eprintln!("factor: {}", e); exit_code = 1; }
            }
        }
    } else {
        for arg in args {
            exit_code |= factor_one(arg, &mut out);
        }
    }
    exit_code
}
