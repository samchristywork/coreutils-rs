use std::io::{self, Write};

pub fn run(args: &[String]) -> i32 {
    let mut utc = false;
    let mut format: Option<String> = None;
    let mut set_date: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            "-u" | "--utc" | "--universal" => utc = true,
            "-s" | "--set" => {
                i += 1;
                if i >= args.len() { eprintln!("date: option requires an argument -- 's'"); return 1; }
                set_date = Some(args[i].clone());
            }
            _ if arg.starts_with("--set=") => {
                set_date = Some(arg["--set=".len()..].to_string());
            }
            _ if arg.starts_with('+') => {
                format = Some(arg[1..].to_string());
            }
            _ if arg.starts_with('-') => {
                eprintln!("date: unrecognized option '{}'", arg);
                return 1;
            }
            _ => {
                eprintln!("date: extra operand '{}'", arg);
                return 1;
            }
        }
        i += 1;
    }

    if let Some(s) = set_date {
        return set_system_date(&s);
    }

    let now = get_time();
    let fmt = format.as_deref().unwrap_or("%a %b %e %H:%M:%S %Z %Y");
    let out = format_date(fmt, now, utc);

    let stdout = io::stdout();
    let mut w = io::BufWriter::new(stdout.lock());
    let _ = writeln!(w, "{}", out);
    0
}
