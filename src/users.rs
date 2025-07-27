use std::collections::BTreeSet;
use std::ffi::CStr;

pub fn run(args: &[String]) -> i32 {
    let mut file: Option<&str> = None;

    for arg in args {
        if arg.starts_with('-') {
            eprintln!("users: unrecognized option '{}'", arg);
            return 1;
        }
        file = Some(arg.as_str());
    }

    let utmp_path = file.unwrap_or("/var/run/utmp");

    let logged_in = match read_utmp(utmp_path) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("users: {}: {}", utmp_path, e);
            return 1;
        }
    };

    if !logged_in.is_empty() {
        println!("{}", logged_in.into_iter().collect::<Vec<_>>().join(" "));
    }
    0
}

// utmp record layout (Linux x86_64)
#[repr(C)]
struct ExitStatus {
    e_termination: i16,
    e_exit: i16,
}

#[repr(C)]
struct Timeval {
    tv_sec: i32,
    tv_usec: i32,
}

#[repr(C)]
struct Utmp {
    ut_type: i16,
    _pad: [u8; 2],
    ut_pid: i32,
    ut_line: [u8; 32],
    ut_id: [u8; 4],
    ut_user: [u8; 32],
    ut_host: [u8; 256],
    ut_exit: ExitStatus,
    ut_session: i32,
    ut_tv: Timeval,
    ut_addr_v6: [i32; 4],
    _unused: [u8; 20],
}
