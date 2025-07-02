use std::fs;
use std::io::{self, Write};
use std::os::unix::io::AsRawFd;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Termios {
    c_iflag: u32,
    c_oflag: u32,
    c_cflag: u32,
    c_lflag: u32,
    c_line: u8,
    c_cc: [u8; 32],
    c_ispeed: u32,
    c_ospeed: u32,
}

extern "C" {
    fn tcgetattr(fd: i32, termios: *mut Termios) -> i32;
    fn tcsetattr(fd: i32, optional_actions: i32, termios: *const Termios) -> i32;
}

impl Termios {
    pub fn get(fd: i32) -> Option<Self> {
        let mut t = std::mem::MaybeUninit::<Termios>::uninit();
        let ret = unsafe { tcgetattr(fd, t.as_mut_ptr()) };
        if ret == 0 { Some(unsafe { t.assume_init() }) } else { None }
    }

    pub fn apply(self, fd: i32) -> Option<()> {
        let ret = unsafe { tcsetattr(fd, 0, &self) }; // TCSANOW = 0
        if ret == 0 { Some(()) } else { None }
    }

    pub fn set_raw(&mut self) {
        self.c_iflag &= !(IGNBRK | BRKINT | PARMRK | ISTRIP | INLCR | IGNCR | ICRNL | IXON);
        self.c_oflag &= !OPOST;
        self.c_lflag &= !(ECHO | ECHONL | ICANON | ISIG | IEXTEN);
        self.c_cflag &= !(CSIZE | PARENB);
        self.c_cflag |= CS8;
        self.c_cc[VMIN] = 1;
        self.c_cc[VTIME] = 0;
    }
}

const IGNBRK: u32 = 0x001;
const BRKINT: u32 = 0x002;
const PARMRK: u32 = 0x008;
const ISTRIP: u32 = 0x020;
const INLCR: u32 = 0x040;
const IGNCR: u32 = 0x080;
const ICRNL: u32 = 0x100;
const IXON: u32 = 0x400;
const OPOST: u32 = 0x001;
const ECHO: u32 = 0x008;
const ECHONL: u32 = 0x040;
const ICANON: u32 = 0x002;
const ISIG: u32 = 0x001;
const IEXTEN: u32 = 0x008000;
const CSIZE: u32 = 0x030;
const PARENB: u32 = 0x100;
const CS8: u32 = 0x030;
const VMIN: usize = 6;
const VTIME: usize = 5;

pub struct Term {
    pub tty: fs::File,
    pub saved: Termios,
    pub last_byte: u8,
    pub last_char: Option<char>,
}

impl Term {
    pub fn open() -> Option<Self> {
        let tty = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/tty")
            .ok()?;
        let saved = Termios::get(tty.as_raw_fd())?;
        let mut raw = saved;
        raw.set_raw();
        raw.apply(tty.as_raw_fd())?;
        Some(Term { tty, saved, last_byte: 0, last_char: None })
    }

    pub fn size(&self) -> (usize, usize) {
        terminal_size(self.tty.as_raw_fd()).unwrap_or((24, 80))
    }

    pub fn read_key(&mut self) -> Key {
        self.last_char = None;
        self.last_byte = 0;
        let mut buf = [0u8; 8];
        let n = match read_fd(self.tty.as_raw_fd(), &mut buf) {
            Ok(n) if n > 0 => n,
            _ => return Key::Unknown,
        };
        self.last_byte = buf[0];

        match &buf[..n] {
            b"q" | b"Q" => Key::Quit,
            b"j" => Key::J,
            b"k" => Key::K,
            b" " => Key::Space,
            b"b" => Key::B,
            b"n" => Key::N,
            b"N" => Key::ShiftN,
            b"/" => Key::Slash,
            b"g" => Key::Home,
            b"G" => Key::ShiftG,
            b"\r" | b"\n" => Key::Enter,
            b"\x1b" => Key::Escape,
            b"\x1b[A" => Key::Up,
            b"\x1b[B" => Key::Down,
            b"\x1b[5~" => Key::PageUp,
            b"\x1b[6~" => Key::PageDown,
            b"\x1b[H" | b"\x1b[1~" => Key::Home,
            b"\x1b[F" | b"\x1b[4~" => Key::End,
            _ => {
                if n == 1 && buf[0] >= 0x20 {
                    self.last_char = Some(buf[0] as char);
                }
                Key::Unknown
            }
        }
    }

    pub fn enter_alt_screen(&mut self) {
        write!(self.tty, "\x1b[?1049h\x1b[?25l").ok();
        self.tty.flush().ok();
    }

    pub fn leave_alt_screen(&mut self) {
        write!(self.tty, "\x1b[?1049l\x1b[?25h").ok();
        self.tty.flush().ok();
    }
}

impl Drop for Term {
    fn drop(&mut self) {
        write!(self.tty, "\x1b[?25h").ok(); // ensure cursor is restored
        self.tty.flush().ok();
        let _ = self.saved.apply(self.tty.as_raw_fd());
    }
}

pub fn is_tty() -> bool {
    extern "C" {
        fn isatty(fd: i32) -> i32;
    }
    unsafe { isatty(1) == 1 }
}

pub fn terminal_size(fd: i32) -> Option<(usize, usize)> {
    #[repr(C)]
    struct Winsize { rows: u16, cols: u16, _x: u16, _y: u16 }
    extern "C" {
        fn ioctl(fd: i32, req: u64, ...) -> i32;
    }
    let mut ws = Winsize { rows: 0, cols: 0, _x: 0, _y: 0 };
    let ret = unsafe { ioctl(fd, 0x5413, &mut ws) };
    if ret == 0 && ws.rows > 0 && ws.cols > 0 {
        Some((ws.rows as usize, ws.cols as usize))
    } else {
        None
    }
}

pub fn read_fd(fd: i32, buf: &mut [u8]) -> io::Result<usize> {
    extern "C" {
        fn read(fd: i32, buf: *mut u8, count: usize) -> isize;
    }
    let n = unsafe { read(fd, buf.as_mut_ptr(), buf.len()) };
    if n < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(n as usize)
    }
}

#[derive(Debug)]
pub enum Key {
    Quit,
    Escape,
    Enter,
    Up, Down,
    J, K,
    Space, B,
    PageUp, PageDown,
    Home, End,
    ShiftG,
    N, ShiftN,
    Slash,
    Unknown,
}
