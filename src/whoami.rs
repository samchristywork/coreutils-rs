use std::ffi::CStr;

pub fn run(args: &[String]) -> i32 {
    for arg in args {
        if arg.starts_with('-') {
            eprintln!("whoami: unrecognized option '{}'", arg);
            return 1;
        }
    }

    #[repr(C)]
    struct Passwd { pw_name: *const i8, _rest: [u8; 64] }
    #[allow(clashing_extern_declarations)]
    extern "C" {
        fn getuid() -> u32;
        fn getpwuid(uid: u32) -> *const Passwd;
    }

    let uid = unsafe { getuid() };
    let pw = unsafe { getpwuid(uid) };
    if pw.is_null() {
        eprintln!("whoami: cannot find name for user ID {}", uid);
        return 1;
    }
    let name = unsafe { CStr::from_ptr((*pw).pw_name).to_string_lossy() };
    println!("{}", name);
    0
}
