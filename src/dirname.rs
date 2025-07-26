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

fn compute(path: &str) -> String {
    // Strip trailing slashes (but keep at least one for root)
    let trimmed = path.trim_end_matches('/');

    if trimmed.is_empty() {
        // path was all slashes
        return "/".to_string();
    }

    match trimmed.rfind('/') {
        None => ".".to_string(),
        Some(0) => "/".to_string(),
        Some(pos) => {
            // Strip any extra trailing slashes before the last component
            let dir = trimmed[..pos].trim_end_matches('/');
            if dir.is_empty() { "/".to_string() } else { dir.to_string() }
        }
    }
}
