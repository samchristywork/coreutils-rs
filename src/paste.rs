use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

pub fn run(args: &[String]) -> i32 {
    let mut delimiter = vec![b'\t'];
    let mut serial = false;
    let mut paths: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            "-s" | "--serial" => serial = true,
            "-d" | "--delimiters" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("paste: option requires an argument -- 'd'");
                    return 1;
                }
                delimiter = parse_delimiters(&args[i]);
            }
            _ if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") => {
                let mut chars = arg[1..].chars().peekable();
                while let Some(ch) = chars.next() {
                    match ch {
                        's' => serial = true,
                        'd' => {
                            let rest: String = chars.collect();
                            let val = if rest.is_empty() {
                                i += 1;
                                if i >= args.len() {
                                    eprintln!("paste: option requires an argument -- 'd'");
                                    return 1;
                                }
                                args[i].clone()
                            } else {
                                rest
                            };
                            delimiter = parse_delimiters(&val);
                            break;
                        }
                        _ => {
                            eprintln!("paste: invalid option -- '{}'", ch);
                            return 1;
                        }
                    }
                }
            }
            _ if arg.starts_with('-') => {
                eprintln!("paste: unrecognized option '{}'", arg);
                return 1;
            }
            _ => paths.push(args[i].clone()),
        }
        i += 1;
    }

    if paths.is_empty() {
        paths.push("-".to_string());
    }

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    if serial {
        paste_serial(&paths, &delimiter, &mut out)
    } else {
        paste_parallel(&paths, &delimiter, &mut out)
    }
}
