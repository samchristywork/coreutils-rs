use std::fs;
use std::io::{self, Read, Write};

use crate::term::{is_tty, Key, Term};

pub fn run(args: &[String]) -> i32 {
    let mut show_line_numbers = false;
    let mut paths: Vec<String> = Vec::new();

    for arg in args {
        if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") {
            for ch in arg[1..].chars() {
                match ch {
                    'N' => show_line_numbers = true,
                    _ => {
                        eprintln!("less: invalid option -- '{}'", ch);
                        return 1;
                    }
                }
            }
        } else {
            match arg.as_str() {
                "--LINE-NUMBERS" => show_line_numbers = true,
                a if a.starts_with('-') => {
                    eprintln!("less: unrecognized option '{}'", a);
                    return 1;
                }
                _ => paths.push(arg.clone()),
            }
        }
    }

    let content = if paths.is_empty() {
        let mut buf = String::new();
        if io::stdin().read_to_string(&mut buf).is_err() {
            eprintln!("less: error reading stdin");
            return 1;
        }
        buf
    } else {
        match fs::read_to_string(&paths[0]) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("less: {}: {}", paths[0], e);
                return 1;
            }
        }
    };

    let lines: Vec<&str> = content.split('\n').collect();
    let lines = if lines.last() == Some(&"") {
        &lines[..lines.len() - 1]
    } else {
        &lines[..]
    };

    if !is_tty() {
        let stdout = io::stdout();
        let mut out = io::BufWriter::new(stdout.lock());
        for line in lines {
            let _ = writeln!(out, "{}", line);
        }
        return 0;
    }

    let mut term = match Term::open() {
        Some(t) => t,
        None => {
            eprintln!("less: could not open terminal");
            return 1;
        }
    };
    term.enter_alt_screen();

    let mut pager = Pager {
        lines,
        top: 0,
        show_line_numbers,
        search: String::new(),
        search_matches: Vec::new(),
        term,
    };

    pager.run()
}

struct Pager<'a> {
    lines: &'a [&'a str],
    top: usize,
    show_line_numbers: bool,
    search: String,
    search_matches: Vec<usize>,
    term: Term,
}

impl<'a> Pager<'a> {
    fn run(&mut self) -> i32 {
        self.draw();

        loop {
            let key = self.term.read_key();
            let (rows, _) = self.term.size();
            let page = rows.saturating_sub(1);

            match key {
                Key::Quit | Key::Escape => break,
                Key::Down | Key::J | Key::Enter => self.scroll_down(1),
                Key::Up | Key::K => self.scroll_up(1),
                Key::PageDown | Key::Space => self.scroll_down(page),
                Key::PageUp | Key::B => self.scroll_up(page),
                Key::Home => self.top = 0,
                Key::End | Key::ShiftG => {
                    let max = self.lines.len().saturating_sub(page);
                    self.top = max;
                }
                Key::Slash => {
                    if let Some(pat) = self.prompt_search() {
                        self.search = pat;
                        self.update_matches();
                        self.jump_to_next_match(self.top);
                    }
                }
                Key::N => self.jump_to_next_match(self.top + 1),
                Key::ShiftN => self.jump_to_prev_match(),
                _ => {}
            }
            self.draw();
        }

        self.term.leave_alt_screen();
        0
    }

    fn scroll_down(&mut self, n: usize) {
        let (rows, _) = self.term.size();
        let max = self.lines.len().saturating_sub(rows.saturating_sub(1));
        self.top = (self.top + n).min(max);
    }

    fn scroll_up(&mut self, n: usize) {
        self.top = self.top.saturating_sub(n);
    }

    fn update_matches(&mut self) {
        self.search_matches.clear();
        if self.search.is_empty() {
            return;
        }
        let pat = self.search.to_lowercase();
        for (i, line) in self.lines.iter().enumerate() {
            if line.to_lowercase().contains(&pat) {
                self.search_matches.push(i);
            }
        }
    }

    fn jump_to_next_match(&mut self, from: usize) {
        if let Some(&m) = self.search_matches.iter().find(|&&m| m >= from) {
            self.top = m;
        } else if let Some(&m) = self.search_matches.first() {
            self.top = m;
        }
    }

    fn jump_to_prev_match(&mut self) {
        if let Some(&m) = self.search_matches.iter().rev().find(|&&m| m < self.top) {
            self.top = m;
        } else if let Some(&m) = self.search_matches.last() {
            self.top = m;
        }
    }

    fn prompt_search(&mut self) -> Option<String> {
        let (rows, _) = self.term.size();
        write!(self.term.tty, "\x1b[{};1H\x1b[2K/", rows).ok()?;
        self.term.tty.flush().ok()?;

        let mut input = String::new();
        loop {
            let raw_byte = {
                let mut b = [0u8];
                use crate::term::read_fd;
                use std::os::unix::io::AsRawFd;
                read_fd(self.term.tty.as_raw_fd(), &mut b).ok();
                b[0]
            };
            match raw_byte {
                b'\r' | b'\n' => break,
                b'\x1b' => return None,
                127 | 8 => {
                    if !input.is_empty() {
                        input.pop();
                        write!(self.term.tty, "\x08 \x08").ok();
                        self.term.tty.flush().ok();
                    }
                }
                b if b >= 0x20 => {
                    input.push(b as char);
                    write!(self.term.tty, "{}", b as char).ok();
                    self.term.tty.flush().ok();
                }
                _ => {}
            }
        }

        if input.is_empty() { None } else { Some(input) }
    }

    fn draw(&mut self) {
    }
}
