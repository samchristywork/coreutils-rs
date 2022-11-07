use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process;

fn cat() {
    let mut args = env::args();
    args.next();
    args.next();

    loop {
        let filename = match args.next() {
            Some(a) => a,
            _ => break,
        };

        let file = File::open(filename).unwrap();
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line.unwrap();
            println!("{}", line);
        }
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

    let util_funcs = HashMap::from([("cat", cat)]);

    match util_funcs.get(util_name.as_str()) {
        Some(util_func) => util_func(),
        None => println!("b"),
    };
}
