mod cp;
mod ls;
mod cat;
mod cmp;
mod comm;
mod cut;
mod diff;
mod join;
mod paste;
mod sort;
mod tr;
mod uniq;
mod head;
mod nl;
mod tail;
mod wc;
mod less;
mod mkdir;
mod more;
mod term;
mod tac;
mod mv;
mod rm;
mod rmdir;

fn main() {
    let mut args: Vec<String> = std::env::args().collect();

    // Support multi-call binary: dispatch based on argv[0] basename
    let prog = args[0]
        .rsplit('/')
        .next()
        .unwrap_or(&args[0])
        .to_string();

    let (cmd, cmd_args) = if prog != "coreutils-rs" {
        (prog, &args[1..])
    } else {
        if args.len() < 2 {
            eprintln!("Usage: coreutils-rs <command> [args...]");
            std::process::exit(1);
        }
        (args.remove(1), &args[1..])
    };

    let code = match cmd.as_str() {
        // File and Directory Operations
        "ls" => ls::run(cmd_args),
        "cp" => cp::run(cmd_args),
        "mv" => mv::run(cmd_args),
        "rm" => rm::run(cmd_args),
        "mkdir" => mkdir::run(cmd_args),
        "rmdir" => rmdir::run(cmd_args),
        // File Viewing and Text Utilities
        "cat" => cat::run(cmd_args),
        "tac" => tac::run(cmd_args),
        "less" => less::run(cmd_args),
        "more" => more::run(cmd_args),
        "head" => head::run(cmd_args),
        "tail" => tail::run(cmd_args),
        "nl" => nl::run(cmd_args),
        "wc" => wc::run(cmd_args),
        // Text Processing
        "comm" => comm::run(cmd_args),
        "join" => join::run(cmd_args),
        "cut" => cut::run(cmd_args),
        "paste" => paste::run(cmd_args),
        "sort" => sort::run(cmd_args),
        "uniq" => uniq::run(cmd_args),
        "tr" => tr::run(cmd_args),
        // Searching and Comparing
        "cmp" => cmp::run(cmd_args),
        "diff" => diff::run(cmd_args),
        _ => {
            eprintln!("coreutils-rs: '{}' is not a supported command", cmd);
            1
        }
    };

    std::process::exit(code);
}
