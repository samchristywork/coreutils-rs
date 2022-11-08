use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::process;

fn cat() {
    let mut args = env::args();
    args.next();
    args.next();

    let mut first = true;
    loop {
        let filename = match args.next() {
            Some(name) => name,
            _ => {
                if first {
                    "/dev/stdin".to_string()
                } else {
                    break;
                }
            }
        };

        let file = File::open(filename).unwrap();
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line.unwrap();
            println!("{}", line);
        }

        first = false;
    }
}

fn cp() {
    let mut args = env::args();
    args.next();
    args.next();

    let source_filename = args.next().unwrap();
    let dest_filename = args.next().unwrap();

    let mut dest = File::create(dest_filename).unwrap();
    let source = File::open(source_filename).unwrap();
    let reader = BufReader::new(source);

    for line in reader.lines() {
        let line = line.unwrap() + "\n";
        dest.write_all(line.as_bytes()).unwrap();
    }
}

fn echo() {
    let mut args = env::args();
    args.next();
    args.next();

    let mut first = true;
    loop {
        let s = match args.next() {
            Some(name) => name,
            _ => break,
        };

        if first {
            print!("{}", s);
        } else {
            print!(" {}", s);
        }
        first = false;
    }
    println!("");
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

fn pwd() {
    let cwd = std::env::current_dir().unwrap();
    println!("{}", cwd.display());
}

fn wc() {
    let mut args = env::args();
    args.next();
    args.next();

    let mut total_lines = 0;
    let mut total_words = 0;
    let mut total_characters = 0;

    let mut first = true;
    loop {
        let filename = match args.next() {
            Some(name) => name,
            _ => {
                if first {
                    "/dev/stdin".to_string()
                } else {
                    break;
                }
            }
        };

        let mut lines = 0;
        let mut words = 0;
        let mut characters = 0;
        let file = File::open(filename.as_str()).unwrap();
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line.unwrap();
            lines += 1;
            for s in line.split(" ") {
                if !s.eq("") {
                    words += 1;
                }
            }
            characters += line.len() + 1;
        }
        println!("{} {} {} {}", lines, words, characters, filename);
        total_lines += lines;
        total_words += words;
        total_characters += characters;

        first = false;
    }

    println!("{} {} {} total", total_lines, total_words, total_characters);
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
    util_funcs.add_func("cat", cat);
    util_funcs.add_func("cp", cp);
    util_funcs.add_func("echo", echo);
    util_funcs.add_func("ls", ls);
    util_funcs.add_func("nproc", nproc);
    util_funcs.add_func("pwd", pwd);
    util_funcs.add_func("wc", wc);

    util_funcs.utils.get(util_name.as_str()).unwrap()();
}
