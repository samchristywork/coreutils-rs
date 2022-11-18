use std::env;
use std::fs;
use std::fs::OpenOptions;

pub fn ls() {
    let mut args = env::args();
    args.next();
    args.next();

    let dir = args.next().unwrap();
    let paths = fs::read_dir(dir).unwrap();
    for path in paths {
        println!("{}", path.unwrap().file_name().to_str().unwrap());
    }
}

pub fn pwd() {
    let cwd = std::env::current_dir().unwrap();
    println!("{}", cwd.display());
}

pub fn truncate() {
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
