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

fn get_time() -> i64 {
    let mut ts = Timespec { tv_sec: 0, tv_nsec: 0 };
    unsafe { clock_gettime(0, &mut ts) };
    ts.tv_sec
}

fn get_tm(secs: i64, utc: bool) -> Tm {
    let mut tm = Tm {
        tm_sec: 0, tm_min: 0, tm_hour: 0,
        tm_mday: 0, tm_mon: 0, tm_year: 0,
        tm_wday: 0, tm_yday: 0, tm_isdst: 0,
        tm_gmtoff: 0, tm_zone: std::ptr::null(),
    };
    unsafe {
        if utc { gmtime_r(&secs, &mut tm); }
        else    { localtime_r(&secs, &mut tm); }
    }
    tm
}

fn format_date(fmt: &str, secs: i64, utc: bool) -> String {
    let tm = get_tm(secs, utc);
    let mut out = String::new();
    let mut chars = fmt.chars().peekable();

    const WEEKDAYS: [&str; 7] = ["Sunday","Monday","Tuesday","Wednesday","Thursday","Friday","Saturday"];
    const WEEKDAYS_ABB: [&str; 7] = ["Sun","Mon","Tue","Wed","Thu","Fri","Sat"];
    const MONTHS: [&str; 12] = ["January","February","March","April","May","June",
                                 "July","August","September","October","November","December"];
    const MONTHS_ABB: [&str; 12] = ["Jan","Feb","Mar","Apr","May","Jun",
                                     "Jul","Aug","Sep","Oct","Nov","Dec"];

    while let Some(ch) = chars.next() {
        if ch != '%' { out.push(ch); continue; }
        match chars.next() {
            Some('Y') => out.push_str(&format!("{:04}", tm.tm_year + 1900)),
            Some('y') => out.push_str(&format!("{:02}", (tm.tm_year + 1900) % 100)),
            Some('m') => out.push_str(&format!("{:02}", tm.tm_mon + 1)),
            Some('d') => out.push_str(&format!("{:02}", tm.tm_mday)),
            Some('e') => out.push_str(&format!("{:2}", tm.tm_mday)),
            Some('H') => out.push_str(&format!("{:02}", tm.tm_hour)),
            Some('M') => out.push_str(&format!("{:02}", tm.tm_min)),
            Some('S') => out.push_str(&format!("{:02}", tm.tm_sec)),
            Some('A') => {
                let w = tm.tm_wday.max(0).min(6) as usize;
                out.push_str(WEEKDAYS[w]);
            }
            Some('a') => {
                let w = tm.tm_wday.max(0).min(6) as usize;
                out.push_str(WEEKDAYS_ABB[w]);
            }
            Some('B') => {
                let m = tm.tm_mon.max(0).min(11) as usize;
                out.push_str(MONTHS[m]);
            }
            Some('b') | Some('h') => {
                let m = tm.tm_mon.max(0).min(11) as usize;
                out.push_str(MONTHS_ABB[m]);
            }
            Some('j') => out.push_str(&format!("{:03}", tm.tm_yday + 1)),
            Some('u') => {
                let w = if tm.tm_wday == 0 { 7 } else { tm.tm_wday };
                out.push_str(&w.to_string());
            }
            Some('w') => out.push_str(&tm.tm_wday.to_string()),
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('%') => out.push('%'),
            Some('s') => out.push_str(&secs.to_string()),
            Some('Z') => {
                if utc {
                    out.push_str("UTC");
                } else if !tm.tm_zone.is_null() {
                    use std::ffi::CStr;
                    let tz = unsafe { CStr::from_ptr(tm.tm_zone).to_string_lossy() };
                    out.push_str(&tz);
                }
            }
            Some('z') => {
                let off = tm.tm_gmtoff;
                let sign = if off >= 0 { '+' } else { '-' };
                let off = off.abs();
                out.push_str(&format!("{}{:02}{:02}", sign, off / 3600, (off % 3600) / 60));
            }
            Some('p') => out.push_str(if tm.tm_hour < 12 { "AM" } else { "PM" }),
            Some('P') => out.push_str(if tm.tm_hour < 12 { "am" } else { "pm" }),
            Some('I') => out.push_str(&format!("{:02}", if tm.tm_hour % 12 == 0 { 12 } else { tm.tm_hour % 12 })),
            Some('l') => out.push_str(&format!("{:2}", if tm.tm_hour % 12 == 0 { 12 } else { tm.tm_hour % 12 })),
            Some('D') => {
                out.push_str(&format!("{:02}/{:02}/{:02}",
                    tm.tm_mon + 1, tm.tm_mday, (tm.tm_year + 1900) % 100));
            }
            Some('F') => {
                out.push_str(&format!("{:04}-{:02}-{:02}",
                    tm.tm_year + 1900, tm.tm_mon + 1, tm.tm_mday));
            }
            Some('T') => {
                out.push_str(&format!("{:02}:{:02}:{:02}",
                    tm.tm_hour, tm.tm_min, tm.tm_sec));
            }
            Some('R') => {
                out.push_str(&format!("{:02}:{:02}", tm.tm_hour, tm.tm_min));
            }
            Some('c') => {
                let w = tm.tm_wday.max(0).min(6) as usize;
                let m = tm.tm_mon.max(0).min(11) as usize;
                out.push_str(&format!("{} {} {:2} {:02}:{:02}:{:02} {:04}",
                    WEEKDAYS_ABB[w], MONTHS_ABB[m], tm.tm_mday,
                    tm.tm_hour, tm.tm_min, tm.tm_sec, tm.tm_year + 1900));
            }
            Some('x') => {
                out.push_str(&format!("{:02}/{:02}/{:02}",
                    tm.tm_mon + 1, tm.tm_mday, (tm.tm_year + 1900) % 100));
            }
            Some('X') => {
                out.push_str(&format!("{:02}:{:02}:{:02}",
                    tm.tm_hour, tm.tm_min, tm.tm_sec));
            }
            Some(c) => { out.push('%'); out.push(c); }
            None => out.push('%'),
        }
    }
    out
}
