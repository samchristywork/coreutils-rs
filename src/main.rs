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
    util_funcs.add_func("dir", filesystem::ls);
    util_funcs.add_func("echo", io::echo);
    util_funcs.add_func("false", miscellaneous::false_fn);
    util_funcs.add_func("head", io::head);
    util_funcs.add_func("hostid", miscellaneous::hostid);
    util_funcs.add_func("ls", filesystem::ls);
    util_funcs.add_func("md5sum", crypto::md5sum);
    util_funcs.add_func("nl", io::nl);
    util_funcs.add_func("nproc", miscellaneous::nproc);
    util_funcs.add_func("printenv", miscellaneous::printenv);
    util_funcs.add_func("pwd", filesystem::pwd);
    util_funcs.add_func("sha512sum", crypto::sha512sum);
    util_funcs.add_func("tail", io::tail);
    util_funcs.add_func("true", miscellaneous::true_fn);
    util_funcs.add_func("truncate", filesystem::truncate);
    util_funcs.add_func("wc", io::wc);
    util_funcs.add_func("yes", io::yes);

    util_funcs.utils.get(util_name.as_str()).unwrap()();
}
