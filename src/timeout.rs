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
                if let Some(ch) = chars.next() {
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

fn run_with_timeout(cmd_name: &str, cmd_args: &[String], duration_secs: f64, signal: i32, preserve_status: bool) -> i32 {
    let mut child = match Command::new(cmd_name).args(cmd_args).spawn() {
        Ok(c) => c,
        Err(e) => { eprintln!("timeout: cannot run '{}': {}", cmd_name, e); return 126; }
    };

    let pid = child.id();
    let timeout_ns = (duration_secs * 1_000_000_000.0) as u64;

    // Poll every 10ms until timeout or child exits
    let poll_ns: u64 = 10_000_000;
    let mut elapsed: u64 = 0;
    let timed_out;

    #[repr(C)]
    struct Timespec { tv_sec: u64, tv_nsec: u32 }
    extern "C" {
        fn nanosleep(req: *const Timespec, rem: *mut Timespec) -> i32;
        fn kill(pid: i32, sig: i32) -> i32;
    }

    loop {
        match child.try_wait() {
            Ok(Some(_)) => { timed_out = false; break; }
            Ok(None) => {}
            Err(_) => { timed_out = false; break; }
        }
        if elapsed >= timeout_ns {
            unsafe { kill(pid as i32, signal) };
            timed_out = true;
            break;
        }
        let req = Timespec { tv_sec: 0, tv_nsec: poll_ns as u32 };
        let mut rem = Timespec { tv_sec: 0, tv_nsec: 0 };
        unsafe { nanosleep(&req, &mut rem) };
        elapsed += poll_ns;
    }

    let status = match child.wait() {
        Ok(s) => s,
        Err(_) => return 1,
    };

    if timed_out {
        if preserve_status {
            return exit_code_from_status(&status);
        }
        return 124;
    }

    exit_code_from_status(&status)
}

fn exit_code_from_status(status: &std::process::ExitStatus) -> i32 {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(sig) = status.signal() {
            return 128 + sig;
        }
    }
    status.code().unwrap_or(1)
}

fn parse_duration(s: &str) -> Option<f64> {
    let trimmed = s.trim_end_matches(|c: char| c.is_alphabetic());
    let suffix = &s[trimmed.len()..];
    let val: f64 = trimmed.parse().ok()?;
    if val < 0.0 { return None; }
    let mult = match suffix {
        "s" | "" => 1.0,
        "m"      => 60.0,
        "h"      => 3600.0,
        "d"      => 86400.0,
        _        => return None,
    };
    Some(val * mult)
}

fn parse_signal(s: &str) -> Option<i32> {
    // Try numeric first
    if let Ok(n) = s.parse::<i32>() { return Some(n); }
    // Strip SIG prefix if present
    let name = s.strip_prefix("SIG").unwrap_or(s).to_uppercase();
    let sig = match name.as_str() {
        "HUP"  => 1,  "INT"  => 2,  "QUIT" => 3,  "ILL"  => 4,
        "TRAP" => 5,  "ABRT" => 6,  "BUS"  => 7,  "FPE"  => 8,
        "KILL" => 9,  "USR1" => 10, "SEGV" => 11, "USR2" => 12,
        "PIPE" => 13, "ALRM" => 14, "TERM" => 15, "CHLD" => 17,
        "CONT" => 18, "STOP" => 19, "TSTP" => 20, "TTIN" => 21,
        "TTOU" => 22, "URG"  => 23, "XCPU" => 24, "XFSZ" => 25,
        "VTALRM"=> 26,"PROF" => 27, "WINCH"=> 28, "IO"   => 29,
        "PWR"  => 30, "SYS"  => 31,
        _ => return None,
    };
    Some(sig)
}
