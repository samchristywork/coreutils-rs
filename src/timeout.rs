use std::process::Command;

pub fn run(args: &[String]) -> i32 {
    let mut kill_after: Option<f64> = None;
    let mut signal = 15i32; // SIGTERM
    let mut preserve_status = false;
    let mut foreground = false;

    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            "--preserve-status" => preserve_status = true,
            "--foreground" => foreground = true,
            "-k" | "--kill-after" => {
                i += 1;
                if i >= args.len() { eprintln!("timeout: option requires an argument -- 'k'"); return 1; }
                match parse_duration(&args[i]) {
                    Some(s) => kill_after = Some(s),
                    None => { eprintln!("timeout: invalid time interval '{}'", args[i]); return 1; }
                }
            }
            "-s" | "--signal" => {
                i += 1;
                if i >= args.len() { eprintln!("timeout: option requires an argument -- 's'"); return 1; }
                match parse_signal(&args[i]) {
                    Some(s) => signal = s,
                    None => { eprintln!("timeout: invalid signal '{}'", args[i]); return 1; }
                }
            }
            _ if arg.starts_with("--kill-after=") => {
                match parse_duration(&arg["--kill-after=".len()..]) {
                    Some(s) => kill_after = Some(s),
                    None => { eprintln!("timeout: invalid time interval"); return 1; }
                }
            }
            _ if arg.starts_with("--signal=") => {
                match parse_signal(&arg["--signal=".len()..]) {
                    Some(s) => signal = s,
                    None => { eprintln!("timeout: invalid signal"); return 1; }
                }
            }
            _ if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") => {
                let mut chars = arg[1..].chars().peekable();
                while let Some(ch) = chars.next() {
                    match ch {
                        'k' => {
                            let rest: String = chars.collect();
                            let val = if rest.is_empty() {
                                i += 1;
                                if i >= args.len() { eprintln!("timeout: option requires an argument -- 'k'"); return 1; }
                                args[i].clone()
                            } else { rest };
                            match parse_duration(&val) {
                                Some(s) => kill_after = Some(s),
                                None => { eprintln!("timeout: invalid time interval '{}'", val); return 1; }
                            }
                            break;
                        }
                        's' => {
                            let rest: String = chars.collect();
                            let val = if rest.is_empty() {
                                i += 1;
                                if i >= args.len() { eprintln!("timeout: option requires an argument -- 's'"); return 1; }
                                args[i].clone()
                            } else { rest };
                            match parse_signal(&val) {
                                Some(s) => signal = s,
                                None => { eprintln!("timeout: invalid signal '{}'", val); return 1; }
                            }
                            break;
                        }
                        _ => { eprintln!("timeout: invalid option -- '{}'", ch); return 1; }
                    }
                }
            }
            _ if arg.starts_with('-') => { eprintln!("timeout: unrecognized option '{}'", arg); return 1; }
            _ => break,
        }
        i += 1;
    }

    if i >= args.len() {
        eprintln!("timeout: missing operand");
        return 1;
    }

    let duration_secs = match parse_duration(&args[i]) {
        Some(s) => s,
        None => { eprintln!("timeout: invalid time interval '{}'", args[i]); return 1; }
    };
    i += 1;

    if i >= args.len() {
        eprintln!("timeout: missing command");
        return 1;
    }

    let cmd_name = &args[i];
    let cmd_args = &args[i+1..];
    let _ = (kill_after, foreground); // may be used in future

    run_with_timeout(cmd_name, cmd_args, duration_secs, signal, preserve_status)
}
