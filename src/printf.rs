use std::io::{self, Write};
use crate::echo::process_escapes;

pub fn run(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("printf: missing operand");
        return 1;
    }

    let fmt = &args[0];
    let mut operands = &args[1..];
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    // Repeat format until all operands consumed; at least one pass
    let mut first = true;
    loop {
        let (output, used) = format_once(fmt, operands);
        let _ = out.write_all(output.as_bytes());
        if used == 0 || operands.is_empty() {
            if first && operands.is_empty() { break; }
            if !first { break; }
        }
        operands = &operands[used.min(operands.len())..];
        first = false;
        if operands.is_empty() { break; }
    }
    0
}
