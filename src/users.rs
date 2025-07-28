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

const USER_PROCESS: i16 = 7;

fn read_utmp(path: &str) -> Result<BTreeSet<String>, std::io::Error> {
    use std::io::Read;
    let mut f = std::fs::File::open(path)?;
    let record_size = std::mem::size_of::<Utmp>();
    let mut buf = vec![0u8; record_size];
    let mut users = BTreeSet::new();

    loop {
        let n = f.read(&mut buf)?;
        if n == 0 { break; }
        if n < record_size { break; }

        let rec: &Utmp = unsafe { &*(buf.as_ptr() as *const Utmp) };
        if rec.ut_type == USER_PROCESS {
            let name = CStr::from_bytes_until_nul(&rec.ut_user)
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
            if !name.is_empty() {
                users.insert(name);
            }
        }
    }
    Ok(users)
}
