use std::ffi::CString;

pub fn run(args: &[String]) -> i32 {
    let mut data_only = false;
    let mut file_system = false;
    let mut paths: Vec<String> = Vec::new();

    for arg in args {
        match arg.as_str() {
            "-d" | "--data" => data_only = true,
            "-f" | "--file-system" => file_system = true,
            a if a.starts_with('-') => { eprintln!("sync: unrecognized option '{}'", a); return 1; }
            _ => paths.push(arg.clone()),
        }
    }

    extern "C" {
        fn sync();
        fn fsync(fd: i32) -> i32;
        fn fdatasync(fd: i32) -> i32;
        fn syncfs(fd: i32) -> i32;
        fn open(path: *const i8, flags: i32, ...) -> i32;
        fn close(fd: i32) -> i32;
    }

    if paths.is_empty() {
        unsafe { sync() };
        return 0;
    }

    let mut exit_code = 0;
    for path in &paths {
        let path_c = match CString::new(path.as_str()) {
            Ok(c) => c,
            Err(_) => { eprintln!("sync: invalid path '{}'", path); exit_code = 1; continue; }
        };
        let fd = unsafe { open(path_c.as_ptr(), 0) }; // O_RDONLY = 0
        if fd < 0 {
            eprintln!("sync: cannot open '{}': {}", path, std::io::Error::last_os_error());
            exit_code = 1;
            continue;
        }
        let ret = unsafe {
            if file_system { syncfs(fd) }
            else if data_only { fdatasync(fd) }
            else { fsync(fd) }
        };
        if ret != 0 {
            eprintln!("sync: error syncing '{}': {}", path, std::io::Error::last_os_error());
            exit_code = 1;
        }
        unsafe { close(fd) };
    }
    exit_code
}
