use std::ffi::CString;
use std::io::{self, Write};

pub fn run(args: &[String]) -> i32 {
    let mut dereference = false;
    let mut format: Option<String> = None;
    let mut filesystem = false;
    let mut paths: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            "-L" | "--dereference" => dereference = true,
            "-f" | "--file-system" => filesystem = true,
            "-c" | "--format" => {
                i += 1;
                if i >= args.len() { eprintln!("stat: option requires an argument -- 'c'"); return 1; }
                format = Some(args[i].clone());
            }
            _ if arg.starts_with("--format=") => {
                format = Some(arg["--format=".len()..].to_string());
            }
            _ if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") => {
                let mut chars = arg[1..].chars().peekable();
                while let Some(ch) = chars.next() {
                    match ch {
                        'L' => dereference = true,
                        'f' => filesystem = true,
                        'c' => {
                            let rest: String = chars.collect();
                            let val = if rest.is_empty() {
                                i += 1;
                                if i >= args.len() { eprintln!("stat: option requires an argument -- 'c'"); return 1; }
                                args[i].clone()
                            } else { rest };
                            format = Some(val);
                            break;
                        }
                        _ => { eprintln!("stat: invalid option -- '{}'", ch); return 1; }
                    }
                }
            }
            _ if arg.starts_with('-') => { eprintln!("stat: unrecognized option '{}'", arg); return 1; }
            _ => paths.push(args[i].clone()),
        }
        i += 1;
    }

    if paths.is_empty() { eprintln!("stat: missing operand"); return 1; }

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let mut exit_code = 0;

    for path in &paths {
        exit_code |= stat_path(path, dereference, filesystem, format.as_deref(), &mut out);
    }
    exit_code
}
