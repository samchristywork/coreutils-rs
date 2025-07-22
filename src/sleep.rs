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
