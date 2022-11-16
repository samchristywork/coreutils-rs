use std::env;
use std::fs::File;
use std::io::Write;
use std::io::{BufRead, BufReader};

pub fn cat() {
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

pub fn cp() {
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

pub fn echo() {
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

pub fn nl() {
    let mut args = env::args();
    args.next();
    args.next();

    let mut idx = 0;
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
            println!("{} {}", idx, line);
            idx += 1;
        }

        first = false;
    }
}

pub fn wc() {
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

pub fn yes() {
    loop {
        println!("y");
    }
}

pub fn head() {
    let mut args = env::args();
    args.next();
    args.next();

    let filename = args.next().unwrap();

    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);

    let mut idx = 0;
    for line in reader.lines() {
        let line = line.unwrap();
        println!("{}", line);
        if idx > 8 {
            break;
        }
        idx += 1;
    }
}

pub fn tail() {
    let mut args = env::args();
    args.next();
    args.next();

    let filename = args.next().unwrap();

    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);

    let mut foo: Vec<String> = vec![];
    for line in reader.lines() {
        let line = line.unwrap();
        foo.push(line);
        if foo.len() > 10 {
            foo.remove(0);
        }
    }

    for line in foo {
        println!("{}", line);
    }
}
