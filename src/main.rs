use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::process;

pub mod crypto;
pub mod io;

extern "C" {
    fn gethostid() -> i64;
}

fn hostid() {
    unsafe {
        println!("{:x}", gethostid() & 0xffffffff);
    }
}

fn ls() {
    let mut args = env::args();
    args.next();
    args.next();

    let dir = args.next().unwrap();
    let paths = fs::read_dir(dir).unwrap();
    for path in paths {
        println!("{}", path.unwrap().file_name().to_str().unwrap());
    }
}

fn nproc() {
    let paths = fs::read_dir("/sys/class/cpuid").unwrap();
    println!("{}", paths.count());
}

fn printenv() {
    let env = env::vars();
    for e in env {
        println!("{}={}", e.0.as_str(), e.1.as_str());
    }
}

fn pwd() {
    let cwd = std::env::current_dir().unwrap();
    println!("{}", cwd.display());
}

fn truncate() {
    let mut args = env::args();
    args.next();
    args.next();

    let mut filename = String::new();
    let mut size = 0;
    let mut sizearg = false;
    for arg in args {
        if arg == "--size" {
            sizearg = true;
            continue;
        }
        if sizearg == true {
            sizearg = false;
            if arg.ends_with("S") {
                let mut chars = arg.chars();
                chars.next_back();
                size = str::parse(chars.as_str()).unwrap();
            } else {
                size = str::parse(arg.as_str()).unwrap();
            }
            continue;
        }
        filename = arg;
    }

    let f = OpenOptions::new().append(true).open(filename).unwrap();
    f.set_len(size).unwrap();
}

fn true_fn() {
    std::process::exit(0);
}

fn false_fn() {
    std::process::exit(1);
}

struct CallbackContainer {
    utils: HashMap<String, fn()>,
}

impl CallbackContainer {
    fn add_func(&mut self, name: &str, func: fn()) {
        self.utils.insert(String::new() + name, func);
    }
}

fn main() {
    let mut args = env::args();
    let program_name = args.next().unwrap();

    let util_name = match args.next() {
        Some(util_name) => util_name,
        _ => {
            eprintln!("Usage: {} [utility]", program_name);
            process::exit(1);
        }
    };

    let mut util_funcs = CallbackContainer {
        utils: HashMap::new(),
    };
    util_funcs.add_func("cat", io::cat);
    util_funcs.add_func("cp", io::cp);
    util_funcs.add_func("dir", ls);
    util_funcs.add_func("echo", io::echo);
    util_funcs.add_func("false", false_fn);
    util_funcs.add_func("head", io::head);
    util_funcs.add_func("hostid", hostid);
    util_funcs.add_func("ls", ls);
    util_funcs.add_func("md5sum", crypto::md5sum);
    util_funcs.add_func("nl", io::nl);
    util_funcs.add_func("nproc", nproc);
    util_funcs.add_func("printenv", printenv);
    util_funcs.add_func("pwd", pwd);
    util_funcs.add_func("sha512sum", crypto::sha512sum);
    util_funcs.add_func("tail", io::tail);
    util_funcs.add_func("true", true_fn);
    util_funcs.add_func("truncate", truncate);
    util_funcs.add_func("wc", io::wc);
    util_funcs.add_func("yes", io::yes);

    util_funcs.utils.get(util_name.as_str()).unwrap()();
}
