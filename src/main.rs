mod basename;
mod cksum;
mod cp;
mod dirname;
mod echo;
mod r#false;
mod printf;
mod realpath;
mod r#test;
mod r#true;
mod yes;
mod date;
mod expr;
mod factor;
mod groups;
mod hashutil;
mod id;
mod ln;
mod ls;
mod md5sum;
mod readlink;
mod seq;
mod sha1sum;
mod sha256sum;
mod sha512sum;
mod sleep;
mod sum;
mod timeout;
mod users;
mod whoami;
mod cat;
mod chmod;
mod chown;
mod cmp;
mod comm;
mod cut;
mod df;
mod diff;
mod du;
mod join;
mod paste;
mod sort;
mod stat;
mod sync;
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
        // Permissions and Ownership
        "chmod" => chmod::run(cmd_args),
        "chown" => chown::run(cmd_args),
        "chgrp" => chown::run_chgrp(cmd_args),
        // Disk and Filesystem
        "df" => df::run(cmd_args),
        "du" => du::run(cmd_args),
        "stat" => stat::run(cmd_args),
        "sync" => sync::run(cmd_args),
        // Date and Time
        "date" => date::run(cmd_args),
        "sleep" => sleep::run(cmd_args),
        "timeout" => timeout::run(cmd_args),
        // Math and Sequences
        "expr" => expr::run(cmd_args),
        "seq" => seq::run(cmd_args),
        "factor" => factor::run(cmd_args),
        // Links
        "ln" => ln::run(cmd_args),
        "readlink" => readlink::run(cmd_args),
        // User and Group Info
        "whoami" => whoami::run(cmd_args),
        "id" => id::run(cmd_args),
        "groups" => groups::run(cmd_args),
        "users" => users::run(cmd_args),
        // Checksums
        "md5sum" => md5sum::run(cmd_args),
        "sha1sum" => sha1sum::run(cmd_args),
        "sha256sum" => sha256sum::run(cmd_args),
        "sha512sum" => sha512sum::run(cmd_args),
        "sum" => sum::run(cmd_args),
        "cksum" => cksum::run(cmd_args),
        // Path Manipulation
        "basename" => basename::run(cmd_args),
        "dirname" => dirname::run(cmd_args),
        "realpath" => realpath::run(cmd_args),
        // Miscellaneous
        "echo" => echo::run(cmd_args),
        "printf" => printf::run(cmd_args),
        "yes" => yes::run(cmd_args),
        "true" => r#true::run(cmd_args),
        "false" => r#false::run(cmd_args),
        "test" => r#test::run(cmd_args),
        "[" => r#test::run_bracket(cmd_args, true),
        _ => {
            eprintln!("coreutils-rs: '{}' is not a supported command", cmd);
            1
        }
    };

    std::process::exit(code);
}
