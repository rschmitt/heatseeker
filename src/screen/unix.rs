#![cfg(not(windows))]

use screen::Key;
use screen::Key::*;
use std::io::{Read, Write};
use std::fs::{File, OpenOptions};
use libc::{c_ushort, c_int, c_ulong};
use std::os::unix::io::AsRawFd;
use std::path::*;
use std::process::Command;
use std::iter::repeat;
use std::cmp::min;
use ansi;

use std::thread;
use std::sync::mpsc::Receiver;
use std::sync::mpsc;

pub struct Screen {
    tty: Terminal,
    original_stty_state: Vec<u8>,
    pub height: u16,
    pub width: u16,
    pub visible_choices: u16,
    pub start_line: u16,
}

impl Screen {
    pub fn open_screen() -> Screen {
        let mut tty = Terminal::open_terminal();
        let current_stty_state = tty.stty(&["-g"]);
        tty.initialize();
        let (cols, rows) = tty.winsize().unwrap();
        let visible_choices = min(20, rows - 1);
        let start_line = rows - visible_choices - 1;
        Screen {
            tty: tty,
            original_stty_state: current_stty_state,
            height: rows,
            width: cols,
            visible_choices: visible_choices,
            start_line: start_line,
        }
    }

    fn restore_tty(&mut self) {
        self.tty.stty(&[&String::from_utf8(self.original_stty_state.clone()).unwrap()]);
    }

    pub fn move_cursor(&mut self, line: u16, column: u16) {
        self.tty.write(&ansi::setpos(line, column));
    }

    pub fn move_cursor_to_bottom(&mut self) {
        let end_line = self.start_line + self.visible_choices;
        self.move_cursor(end_line, 0);
        self.write("\n");
    }

    pub fn blank_screen(&mut self) {
        let start_line = self.start_line;
        self.move_cursor(start_line, 0);
        let blank_line = repeat(' ').take(self.width as usize).collect::<String>();
        for _ in 0..self.height {
            self.tty.write(blank_line.as_bytes());
        }
        self.move_cursor(start_line, 0);
    }

    pub fn show_cursor(&mut self) {
        self.tty.write(&ansi::show_cursor());
    }

    pub fn hide_cursor(&mut self) {
        self.tty.write(&ansi::hide_cursor());
    }

    pub fn write(&mut self, s: &str) {
        self.tty.write(s.as_bytes());
    }

    pub fn write_red_inverted(&mut self, s: &str) {
        self.tty.write(&ansi::red());
        self.tty.write(&ansi::inverse());
        self.tty.write(s.as_bytes());
        self.tty.write(&ansi::reset());
    }

    pub fn write_red(&mut self, s: &str) {
        self.tty.write(&ansi::red());
        self.tty.write(s.as_bytes());
        self.tty.write(&ansi::reset());
    }

    pub fn write_inverted(&mut self, s: &str) {
        self.tty.write(&ansi::inverse());
        self.tty.write(s.as_bytes());
        self.tty.write(&ansi::reset());
    }

    // Return all buffered keystrokes, or the next key if buffer is empty.
    pub fn get_buffered_keys(&mut self) -> Vec<Key> {
        let mut ret = Vec::new();
        while let Ok(byte) = self.tty.input.try_recv() {
            ret.push(byte);
        }
        while ret.is_empty() || Screen::more_bytes_needed(&ret) {
            ret.push(self.tty.input.recv().unwrap());
        }
        Terminal::translate_bytes(ret.clone())
    }

    fn more_bytes_needed(bytes: &Vec<u8>) -> bool {
        if let Err(_) = String::from_utf8(bytes.clone()) {
            return true;
        }
        false
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        self.restore_tty();
    }
}

struct Terminal {
    input: Receiver<u8>,
    input_fd: i32,
    output: File,
}

impl Terminal {
    fn open_terminal() -> Terminal {
        let term_path = Path::new("/dev/tty");
        let mut input_file = File::open(&term_path).unwrap();
        let output_file = OpenOptions::new().write(true).open(&term_path).unwrap();
        let input_fd = input_file.as_raw_fd();
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            loop {
                let mut buf = [0];
                if input_file.read(&mut buf).unwrap() != 1 {
                    panic!("Failed to read a single byte from tty");
                }
                tx.send(buf[0]).unwrap();
            }
        });
        Terminal {
            input: rx,
            input_fd: input_fd,
            output: output_file
        }
    }

    fn initialize(&mut self) {
        self.stty(&["raw", "-echo", "cbreak", "opost"]);
    }

    fn stty(&mut self, args: &[&str]) -> Vec<u8> {
        let mut process = match Command::new("stty")
            .args(args)
            // .stdin(InheritFd(self.input_fd))
            .spawn()
        {
            Ok(p) => p,
            Err(e) => panic!("Spawn failed: {}", e),
        };

        let exit = process.wait();

        let mut buf = Vec::new();
        if exit.unwrap().success() {
            process.stdout.as_mut().unwrap().read_to_end(&mut buf).unwrap();
            let mut str = String::from_utf8(buf).unwrap();

            // The output from `stty -g` may include a newline, which we have to strip off. Otherwise,
            // when we go to restore the tty, stty (on some platforms) will fail with an "invalid
            // argument" error.
            ::trim(&mut str);

            str.into_bytes()
        } else {
            process.stderr.as_mut().unwrap().read_to_end(&mut buf).unwrap();
            panic!(String::from_utf8(buf).unwrap());
        }
    }

    fn translate_bytes(bytes: Vec<u8>) -> Vec<Key> {
        let chars = String::from_utf8(bytes).unwrap().chars().collect::<Vec<char>>();
        chars.into_iter().map(|c| Terminal::translate_char(c)).collect()
    }

    fn translate_char(c: char) -> Key {
        let numeric_char = c as u32;
        if c == '\r' {
            Enter
        } else if numeric_char == 9 {
            Tab
        } else if numeric_char == 127 {
            Backspace
        } else if numeric_char & 96 == 0 && numeric_char <= 128u32 {
            let c = c as u8;
            Control((c + 96u8) as char)
        } else {
            Char(c)
        }
    }

    fn write(&mut self, s: &[u8]) {
        self.output.write_all(&s).unwrap();
    }

    fn writeln(&mut self, s: &str) {
        self.output.write(s.as_bytes()).unwrap();
        self.output.write("\n".as_bytes()).unwrap();
    }

    fn winsize(&self) -> Option<(u16, u16)> {
        extern { fn ioctl(fd: c_int, request: c_ulong, ...) -> c_int; }
        #[cfg(any(target_os = "macos", target_os = "freebsd"))]
        const TIOCGWINSZ: c_ulong = 0x40087468;

        #[cfg(any(target_os = "linux", target_os = "android"))]
        const TIOCGWINSZ: c_ulong = 0x00005413;

        #[repr(C)]
        struct TermSize {
            rows: c_ushort,
            cols: c_ushort,
            x: c_ushort,
            y: c_ushort,
        }

        let size = TermSize { rows: 0, cols: 0, x: 0, y: 0 };
        if unsafe { ioctl(self.output.as_raw_fd(), TIOCGWINSZ, &size) } == 0 {
            Some((size.cols as u16, size.rows as u16))
        } else {
            None
        }
    }
}

#[test]
fn winsize_test() {
    // Travis-CI builds run without a tty, making this test impossible.
    if option_env!("TRAVIS").is_some() {
        // TODO: It should be made obvious from the output that this test was skipped
        return;
    }
    let term = Terminal::open_terminal();
    let (cols, rows) = term.winsize().expect("Failed to get window size!");
    // We don't know the window size a priori, but we can at least
    // assert that it is within some kind of sensible range.
    assert!(cols > 40);
    assert!(rows > 40);
    assert!(cols < 1000);
    assert!(rows < 1000);
}
