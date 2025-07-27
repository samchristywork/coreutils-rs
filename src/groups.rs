use std::ffi::{CStr, CString};

pub fn run(args: &[String]) -> i32 {
    let mut usernames: Vec<&str> = Vec::new();

    for arg in args {
        if arg.starts_with('-') {
            eprintln!("groups: unrecognized option '{}'", arg);
            return 1;
        }
        usernames.push(arg.as_str());
    }

    #[repr(C)]
    struct Passwd {
        pw_name: *const i8,
        pw_passwd: *const i8,
        pw_uid: u32,
        pw_gid: u32,
        pw_gecos: *const i8,
        pw_dir: *const i8,
        pw_shell: *const i8,
    }
    #[repr(C)]
    struct Group { gr_name: *const i8, _rest: [u8; 64] }

    extern "C" {
        fn getuid() -> u32;
        fn getpwuid(uid: u32) -> *const Passwd;
        fn getpwnam(name: *const i8) -> *const Passwd;
        fn getgrgid(gid: u32) -> *const Group;
        fn getgroups(size: i32, list: *mut u32) -> i32;
        fn getgrouplist(user: *const i8, gid: u32, groups: *mut u32, ngroups: *mut i32) -> i32;
    }

    let gid_name = |gid: u32| -> String {
        let gr = unsafe { getgrgid(gid) };
        if gr.is_null() { return gid.to_string(); }
        unsafe { CStr::from_ptr((*gr).gr_name).to_string_lossy().into_owned() }
    };

    let mut exit_code = 0;

    if usernames.is_empty() {
        // Current user's groups
        let mut buf = vec![0u32; 64];
        let n = unsafe { getgroups(buf.len() as i32, buf.as_mut_ptr()) };
        let groups = if n >= 0 {
            buf[..n as usize].to_vec()
        } else {
            let uid = unsafe { getuid() };
            let pw = unsafe { getpwuid(uid) };
            if pw.is_null() { return 1; }
            vec![unsafe { (*pw).pw_gid }]
        };
        let names: Vec<String> = groups.iter().map(|&g| gid_name(g)).collect();
        println!("{}", names.join(" "));
    } else {
        for username in &usernames {
            let c_name = match CString::new(*username) { Ok(c) => c, Err(_) => { exit_code = 1; continue; } };
            let pw = unsafe { getpwnam(c_name.as_ptr()) };
            if pw.is_null() {
                eprintln!("groups: '{}': no such user", username);
                exit_code = 1;
                continue;
            }
            let pw_name = unsafe { (*pw).pw_name };
            let gid = unsafe { (*pw).pw_gid };
            let mut ngroups = 64i32;
            let mut buf = vec![0u32; 64];
            unsafe { getgrouplist(pw_name, gid, buf.as_mut_ptr(), &mut ngroups) };
            buf.truncate(ngroups.max(0) as usize);

            let names: Vec<String> = buf.iter().map(|&g| gid_name(g)).collect();
            if usernames.len() > 1 {
                println!("{} : {}", username, names.join(" "));
            } else {
                println!("{}", names.join(" "));
            }
        }
    }
    exit_code
}
