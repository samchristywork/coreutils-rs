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

#[repr(C)]
struct Timespec {
    tv_sec: i64,
    tv_nsec: i64,
}

#[repr(C)]
struct Tm {
    tm_sec: i32,
    tm_min: i32,
    tm_hour: i32,
    tm_mday: i32,
    tm_mon: i32,
    tm_year: i32,
    tm_wday: i32,
    tm_yday: i32,
    tm_isdst: i32,
    tm_gmtoff: i64,
    tm_zone: *const i8,
}

extern "C" {
    fn clock_gettime(clk_id: i32, tp: *mut Timespec) -> i32;
    fn localtime_r(timep: *const i64, result: *mut Tm) -> *mut Tm;
    fn gmtime_r(timep: *const i64, result: *mut Tm) -> *mut Tm;
    fn mktime(tm: *mut Tm) -> i64;
    fn settimeofday(tv: *const Timeval, tz: *const u8) -> i32;
}

#[repr(C)]
struct Timeval {
    tv_sec: i64,
    tv_usec: i64,
}
