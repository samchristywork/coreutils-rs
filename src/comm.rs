use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

pub fn run(args: &[String]) -> i32 {
    let mut suppress1 = false;
    let mut suppress2 = false;
    let mut suppress3 = false;
    let mut ignore_case = false;
    let mut output_delimiter = "\t".to_string();
    let mut paths: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            "-1" => suppress1 = true,
            "-2" => suppress2 = true,
            "-3" => suppress3 = true,
            "-i" | "--nocheck-order" => ignore_case = true,
            "--output-delimiter" => {
                i += 1;
                if i >= args.len() { eprintln!("comm: option requires an argument -- 'output-delimiter'"); return 1; }
                output_delimiter = args[i].clone();
            }
            _ if arg.starts_with("--output-delimiter=") => {
                output_delimiter = arg["--output-delimiter=".len()..].to_string();
            }
            _ if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") => {
                for ch in arg[1..].chars() {
                    match ch {
                        '1' => suppress1 = true,
                        '2' => suppress2 = true,
                        '3' => suppress3 = true,
                        'i' => ignore_case = true,
                        _ => { eprintln!("comm: invalid option -- '{}'", ch); return 1; }
                    }
                }
            }
            _ if arg.starts_with('-') => { eprintln!("comm: unrecognized option '{}'", arg); return 1; }
            _ => paths.push(args[i].clone()),
        }
        i += 1;
    }

    if paths.len() < 2 {
        eprintln!("comm: missing operand");
        eprintln!("Usage: comm [OPTION]... FILE1 FILE2");
        return 1;
    }

    let mut reader1 = match open(&paths[0]) {
        Some(r) => r,
        None => return 1,
    };
    let mut reader2 = match open(&paths[1]) {
        Some(r) => r,
        None => return 1,
    };

    // Column indents: col1 = no indent, col2 = one delimiter, col3 = two delimiters
    let col2_prefix = output_delimiter.clone();
    let col3_prefix = format!("{}{}", output_delimiter, output_delimiter);

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    let mut line1 = String::new();
    let mut line2 = String::new();
    let mut have1 = read_line(&mut reader1, &mut line1);
    let mut have2 = read_line(&mut reader2, &mut line2);

    loop {
        match (have1, have2) {
            (false, false) => break,
            (true, false) => {
                if !suppress1 {
                    let _ = writeln!(out, "{}", line1.trim_end_matches('\n').trim_end_matches('\r'));
                }
                have1 = read_line(&mut reader1, &mut line1);
            }
            (false, true) => {
                if !suppress2 {
                    let _ = writeln!(out, "{}{}", col2_prefix, line2.trim_end_matches('\n').trim_end_matches('\r'));
                }
                have2 = read_line(&mut reader2, &mut line2);
            }
            (true, true) => {
                let l1 = line1.trim_end_matches('\n').trim_end_matches('\r');
                let l2 = line2.trim_end_matches('\n').trim_end_matches('\r');
                let ord = if ignore_case {
                    l1.to_lowercase().cmp(&l2.to_lowercase())
                } else {
                    l1.cmp(l2)
                };
                match ord {
                    std::cmp::Ordering::Less => {
                        if !suppress1 {
                            let _ = writeln!(out, "{}", l1);
                        }
                        have1 = read_line(&mut reader1, &mut line1);
                    }
                    std::cmp::Ordering::Greater => {
                        if !suppress2 {
                            let _ = writeln!(out, "{}{}", col2_prefix, l2);
                        }
                        have2 = read_line(&mut reader2, &mut line2);
                    }
                    std::cmp::Ordering::Equal => {
                        if !suppress3 {
                            let _ = writeln!(out, "{}{}", col3_prefix, l1);
                        }
                        have1 = read_line(&mut reader1, &mut line1);
                        have2 = read_line(&mut reader2, &mut line2);
                    }
                }
            }
        }
    }

    0
}
