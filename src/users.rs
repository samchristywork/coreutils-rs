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
