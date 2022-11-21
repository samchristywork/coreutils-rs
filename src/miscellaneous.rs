use std::env;
use std::fs;

extern "C" {
    fn gethostid() -> i64;
}

pub fn hostid() {
    unsafe {
        println!("{:x}", gethostid() & 0xffffffff);
    }
}

pub fn nproc() {
    let paths = fs::read_dir("/sys/class/cpuid").unwrap();
    println!("{}", paths.count());
}

pub fn printenv() {
    let env = env::vars();
    for e in env {
        println!("{}={}", e.0.as_str(), e.1.as_str());
    }
}

pub fn true_fn() {
    std::process::exit(0);
}

pub fn false_fn() {
    std::process::exit(1);
}
