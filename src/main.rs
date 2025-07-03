mod cp;
mod ls;
mod mkdir;
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
        "cp" => cp::run(cmd_args),
        "mkdir" => mkdir::run(cmd_args),
        "mv" => mv::run(cmd_args),
        "rm" => rm::run(cmd_args),
        "rmdir" => rmdir::run(cmd_args),
        "ls" => ls::run(cmd_args),
        _ => {
            eprintln!("coreutils-rs: '{}' is not a supported command", cmd);
            1
        }
    };

    std::process::exit(code);
}
