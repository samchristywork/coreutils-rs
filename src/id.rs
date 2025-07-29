use std::ffi::CStr;

pub fn run(args: &[String]) -> i32 {
    let mut only_user = false;
    let mut only_group = false;
    let mut only_groups = false;
    let mut name_not_number = false;
    let mut real = false;
    let mut usernames: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            "-u" | "--user"   => only_user = true,
            "-g" | "--group"  => only_group = true,
            "-G" | "--groups" => only_groups = true,
            "-n" | "--name"   => name_not_number = true,
            "-r" | "--real"   => real = true,
            _ if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") => {
                for ch in arg[1..].chars() {
                    match ch {
                        'u' => only_user = true,
                        'g' => only_group = true,
                        'G' => only_groups = true,
                        'n' => name_not_number = true,
                        'r' => real = true,
                        _ => { eprintln!("id: invalid option -- '{}'", ch); return 1; }
                    }
                }
            }
            _ if arg.starts_with('-') => { eprintln!("id: unrecognized option '{}'", arg); return 1; }
            _ => usernames.push(arg.to_string()),
        }
        i += 1;
    }

    // Validate flag combinations
    let mode_count = only_user as u8 + only_group as u8 + only_groups as u8;
    if mode_count > 1 {
        eprintln!("id: cannot print \"only\" of more than one choice");
        return 1;
    }
    if name_not_number && mode_count == 0 {
        eprintln!("id: option --name (-n) only defined when printing a single ID");
        return 1;
    }

    let target = if usernames.is_empty() {
        None
    } else {
        Some(usernames[0].as_str())
    };

    let info = match get_user_info(target) {
        Some(i) => i,
        None => {
            eprintln!("id: '{}': no such user", usernames[0]);
            return 1;
        }
    };

    let (uid, gid) = if real || target.is_some() {
        (info.uid, info.gid)
    } else {
        (info.euid, info.egid)
    };

    if only_user {
        if name_not_number { println!("{}", uid_name(uid)); }
        else               { println!("{}", uid); }
        return 0;
    }
    if only_group {
        if name_not_number { println!("{}", gid_name(gid)); }
        else               { println!("{}", gid); }
        return 0;
    }
    if only_groups {
        let parts: Vec<String> = info.groups.iter().map(|&g| {
            if name_not_number { gid_name(g) } else { g.to_string() }
        }).collect();
        println!("{}", parts.join(" "));
        return 0;
    }

    // Default: full output
    let uname = uid_name(info.uid);
    let euname = uid_name(info.euid);
    let gname = gid_name(info.gid);
    let egname = gid_name(info.egid);

    let mut out = format!("uid={}({}) gid={}({})", info.uid, uname, info.gid, gname);
    if info.euid != info.uid {
        out.push_str(&format!(" euid={}({})", info.euid, euname));
    }
    if info.egid != info.gid {
        out.push_str(&format!(" egid={}({})", info.egid, egname));
    }
    let groups_str: Vec<String> = info.groups.iter()
        .map(|&g| format!("{}({})", g, gid_name(g)))
        .collect();
    out.push_str(&format!(" groups={}", groups_str.join(",")));
    println!("{}", out);
    0
}
