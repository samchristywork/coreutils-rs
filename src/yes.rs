use std::io::{self, Write};

pub fn run(args: &[String]) -> i32 {
    for arg in args {
        if arg == "--help" { println!("Usage: yes [STRING]..."); return 0; }
        if arg == "--version" { println!("yes (coreutils-rs)"); return 0; }
    }

    let line = if args.is_empty() {
        "y\n".to_string()
    } else {
        format!("{}\n", args.join(" "))
    };

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let bytes = line.as_bytes();

    // Fill a large buffer for efficiency
    let buf_size = 8192;
    let repeat = (buf_size / bytes.len()).max(1);
    let buf: Vec<u8> = bytes.iter().cloned().cycle().take(bytes.len() * repeat).collect();

    loop {
        if out.write_all(&buf).is_err() { break; }
    }
    0
}
