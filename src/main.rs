use std::collections::HashMap;
use std::env;
use std::process;

pub mod crypto;
pub mod filesystem;
pub mod io;
pub mod miscellaneous;

struct CallbackContainer {
    utils: HashMap<String, fn()>,
}

impl CallbackContainer {
    fn add_func(&mut self, func: fn(), name: &str) {
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
    util_funcs.add_func(crypto::md5sum, "md5sum");
    util_funcs.add_func(crypto::sha512sum, "sha512sum");
    util_funcs.add_func(filesystem::ls, "dir");
    util_funcs.add_func(filesystem::ls, "ls");
    util_funcs.add_func(filesystem::pwd, "pwd");
    util_funcs.add_func(filesystem::truncate, "truncate");
    util_funcs.add_func(io::cat, "cat");
    util_funcs.add_func(io::cp, "cp");
    util_funcs.add_func(io::echo, "echo");
    util_funcs.add_func(io::head, "head");
    util_funcs.add_func(io::nl, "nl");
    util_funcs.add_func(io::tail, "tail");
    util_funcs.add_func(io::wc, "wc");
    util_funcs.add_func(io::yes, "yes");
    util_funcs.add_func(miscellaneous::false_fn, "false");
    util_funcs.add_func(miscellaneous::hostid, "hostid");
    util_funcs.add_func(miscellaneous::nproc, "nproc");
    util_funcs.add_func(miscellaneous::printenv, "printenv");
    util_funcs.add_func(miscellaneous::true_fn, "true");

    util_funcs.utils.get(util_name.as_str()).unwrap()();
}
