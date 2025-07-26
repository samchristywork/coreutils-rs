use std::io::{self, Write};

pub fn run(args: &[String]) -> i32 {
    let mut zero = false;
    let mut paths: Vec<String> = Vec::new();

    for arg in args {
        match arg.as_str() {
            "-z" | "--zero" => zero = true,
            _ if arg.starts_with('-') => { eprintln!("dirname: unrecognized option '{}'", arg); return 1; }
            _ => paths.push(arg.clone()),
        }
    }

    if paths.is_empty() {
        eprintln!("dirname: missing operand");
        return 1;
    }

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    for path in &paths {
        let result = compute(path);
        if zero {
            let _ = write!(out, "{}\0", result);
        } else {
            let _ = writeln!(out, "{}", result);
        }
    }
    0
}
