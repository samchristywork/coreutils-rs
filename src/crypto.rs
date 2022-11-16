use md5;
use sha2::{Digest, Sha512};
use std::env;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn md5sum() {
    let mut args = env::args();
    args.next();
    args.next();

    let filename = args.next().unwrap();
    let contents = fs::read_to_string(filename.as_str());
    let digest = md5::compute(contents.unwrap().as_bytes());
    println!("{:x} {}", digest, filename);
}

pub fn sha512sum() {
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

        let file = File::open(filename.as_str()).unwrap();
        let reader = BufReader::new(file);

        let mut hasher = Sha512::new();
        for line in reader.lines() {
            let line = line.unwrap();
            hasher.update(line + "\n");
        }
        let result = hasher.finalize();
        println!("{:x} {}", result, filename.as_str());

        first = false;
    }
}
