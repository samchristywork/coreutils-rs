pub fn run(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("sleep: missing operand");
        return 1;
    }

    let mut total_ns: u128 = 0;

    for arg in args {
        match parse_duration(arg) {
            Some(ns) => total_ns += ns,
            None => {
                eprintln!("sleep: invalid time interval '{}'", arg);
                return 1;
            }
        }
    }

    let secs = (total_ns / 1_000_000_000) as u64;
    let nsecs = (total_ns % 1_000_000_000) as u32;

    #[repr(C)]
    struct Timespec { tv_sec: u64, tv_nsec: u32 }
    extern "C" { fn nanosleep(req: *const Timespec, rem: *mut Timespec) -> i32; }

    let req = Timespec { tv_sec: secs, tv_nsec: nsecs };
    let mut rem = Timespec { tv_sec: 0, tv_nsec: 0 };
    unsafe { nanosleep(&req, &mut rem) };
    0
}

fn parse_duration(s: &str) -> Option<u128> {
    let (num_str, suffix) = split_suffix(s);
    let val: f64 = num_str.parse().ok()?;
    if val < 0.0 { return None; }
    let ns_per_unit: f64 = match suffix {
        "s" | "" => 1_000_000_000.0,
        "m"      => 60.0 * 1_000_000_000.0,
        "h"      => 3600.0 * 1_000_000_000.0,
        "d"      => 86400.0 * 1_000_000_000.0,
        _        => return None,
    };
    Some((val * ns_per_unit) as u128)
}

fn split_suffix(s: &str) -> (&str, &str) {
    let trimmed = s.trim_end_matches(|c: char| c.is_alphabetic());
    let suffix = &s[trimmed.len()..];
    (trimmed, suffix)
}
