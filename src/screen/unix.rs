#![cfg(not(windows))]

extern crate libc;
extern crate signal_hook;

use screen::Key;
use screen::Key::*;
use std::io::{Read, Write};
use std::fs::{File, OpenOptions};
use std::os::unix::io::*;
use std::path::*;
use std::process::Command;
use std::process::Stdio;
use std::str;
use std::iter::repeat;
use std::cmp::min;
use ansi;
use ::NEWLINE;

use std::mem;
use std::thread;
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
use std::sync::{mpsc, Arc, Mutex};

use self::libc::{dup, SIGINT, SIGWINCH, c_int, c_ushort, c_ulong};
use screen::Screen;

pub struct UnixScreen {
    tty: Terminal,
    start_line: u16,
    desired_rows: u16,
}

impl Screen for UnixScreen {
    fn visible_choices(&self) -> u16 {
        let (_, rows) = self.tty.winsize().unwrap();
        min(self.desired_rows, rows - 1)
    }

    fn width(&self) -> u16 {
        let (cols, _) = self.tty.winsize().unwrap();
        cols
    }

    fn move_cursor_to_prompt_line(&mut self, col: u16) {
        self.reset_cursor();
        self.tty.write(&ansi::cursor_right(col));
    }

    fn blank_screen(&mut self) {
        self.reset_cursor();
        let blank_line = repeat(' ').take(self.width() as usize).collect::<String>();
        for _ in 0..self.visible_choices() + 1 {
            self.tty.write(blank_line.as_bytes());
        }
        self.reset_cursor();
    }

    fn show_cursor(&mut self) {
        self.tty.write(&ansi::show_cursor());
        self.tty.flush();
    }

    fn hide_cursor(&mut self) {
        self.tty.write(&ansi::hide_cursor());
    }

    fn write(&mut self, s: &str) {
        self.tty.write(s.as_bytes());
    }

    fn write_red_inverted(&mut self, s: &str) {
        self.tty.write(&ansi::red());
        self.tty.write(&ansi::inverse());
        self.tty.write(s.as_bytes());
        self.tty.write(&ansi::reset());
    }

    fn write_red(&mut self, s: &str) {
        self.tty.write(&ansi::red());
        self.tty.write(s.as_bytes());
        self.tty.write(&ansi::reset());
    }

    fn write_inverted(&mut self, s: &str) {
        self.tty.write(&ansi::inverse());
        self.tty.write(s.as_bytes());
        self.tty.write(&ansi::reset());
    }

    // Return all buffered keystrokes, or the next key if buffer is empty.
    fn get_buffered_keys(&mut self) -> Vec<Key> {
        let mut ret = Vec::new();
        while let Ok(bytes) = self.tty.input.try_recv() {
            ret.extend(bytes);
        }
        while ret.is_empty() {
            let bytes = self.tty.input.recv().unwrap();
            if bytes == vec![SIGWINCH as u8] {
                self.blank_entire_screen();
                return vec![Nothing];
            } else if bytes == vec![SIGINT as u8] {
                return vec![Control('g')];
            } else {
                ret.extend(bytes);
            }
        }
        Terminal::translate_bytes(ret.clone())
    }
}

impl UnixScreen {
    pub fn is_cygwin() -> bool {
        false
    }

    pub fn open_screen(desired_rows: u16) -> UnixScreen {
        let mut tty = Terminal::open_terminal();
        let (_, rows) = tty.winsize().unwrap();
        let visible_choices = min(desired_rows, rows - 1);
        let start_line = rows - visible_choices - 1;
        for _ in 0..visible_choices {
            tty.write(NEWLINE.as_bytes());
        }
        tty.write(&ansi::save_cursor());

        UnixScreen {
            tty,
            start_line,
            desired_rows,
        }
    }

    fn restore_tty(&mut self) {
        self.tty.restore_tty();
    }

    fn reset_cursor(&mut self) {
        self.tty.write(&ansi::restore_cursor());

        // Writing this carriage return works around a rendering bug in
        // Neovim's terminal emulation. Without it, the cursor flies
        // around all over the place while typing, and the input prompt
        // is not rendered correctly. I have no idea why this workaround
        // is effective, or why others are not, or what the root cause
        // of the issue is (likely something involving save/restore
        // support).
        self.tty.write(b"\r");

        let num_lines = self.visible_choices();
        self.tty.write(&ansi::cursor_up(num_lines));
    }

    pub fn blank_entire_screen(&mut self){
        self.tty.write(&ansi::blank_screen());
    }

    fn more_bytes_needed(bytes: &[u8]) -> bool {
        if let Err(_) = str::from_utf8(bytes) { true } else { false }
    }
}

struct Terminal {
    input: Receiver<Vec<u8>>,
    input_fd: RawFd,
    output: File,
    output_buf: Vec<u8>,
    original_stty_state: Vec<u8>,
}

static mut GLOBAL_TX: *const Arc<Mutex<Sender<Vec<u8>>>> = 0 as *const Arc<Mutex<Sender<Vec<u8>>>>;

fn set_global_tx(tx: Sender<Vec<u8>>) {
    unsafe {
        let singleton = Arc::new(Mutex::new(tx));
        GLOBAL_TX = mem::transmute(Box::new(singleton));
    }
}

fn get_global_tx() -> Sender<Vec<u8>> {
    let singleton = unsafe { (*GLOBAL_TX).clone() };
    let tx = singleton.lock().unwrap();
    tx.clone()
}

fn start_sigwinch_handler() {
    let signals = signal_hook::iterator::Signals::new(&[SIGWINCH, SIGINT]).unwrap();
    thread::spawn(move || {
        for signal in signals.forever() {
            get_global_tx().send(vec![signal as u8]).unwrap();
        }
    });
}

impl Terminal {
    fn open_terminal() -> Terminal {
        let term_path = Path::new("/dev/tty");
        let mut input_file = File::open(&term_path).unwrap();
        let output_file = OpenOptions::new().write(true).open(&term_path).unwrap();
        let input_fd = input_file.as_raw_fd();
        let (tx, rx) = mpsc::channel();
        set_global_tx(tx);

        start_sigwinch_handler();

        let mut ret = Terminal {
            input: rx,
            input_fd,
            output: output_file,
            output_buf: Vec::new(),
            original_stty_state: Vec::new(),
        };
        let current_stty_state = ret.stty(&["-g"]);
        ret.original_stty_state = current_stty_state;
        ret.initialize();

        thread::spawn(move || {
            loop {
                let mut buf = [0; 255];
                let tx = get_global_tx();
                if let Ok(length) = input_file.read(&mut buf) {
                    tx.send(buf[0..length].to_vec()).unwrap();
                } else {
                    tx.send([0].to_vec()).unwrap();
                }
            }
        });
        ret
    }

    fn initialize(&mut self) {
        self.stty(&["raw", "-echo", "cbreak", "opost", "onlcr"]);
    }

    fn restore_tty(&mut self) {
        let state = String::from_utf8(self.original_stty_state.clone()).unwrap();
        self.stty(&[&state]);
    }

    fn stty(&mut self, args: &[&str]) -> Vec<u8> {
        let tty_input = unsafe { Stdio::from_raw_fd(dup(self.input_fd)) };
        let mut process = match Command::new("/bin/stty")
            .args(args)
            .stdin(tty_input)
            .stdout(Stdio::piped())
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
        const BEGIN_PASTE: &'static [u8] = b"\x1B[200~";
        const END_PASTE: &'static [u8] = b"\x1B[201~";

        if bytes == b"\x1B[A" || bytes == b"\x1BOA" { return vec![Up] };
        if bytes == b"\x1B[B" || bytes == b"\x1BOB" { return vec![Down] };
        if bytes == b"\x1B[5~" { return vec![PgUp] }
        if bytes == b"\x1B[6~" { return vec![PgDown] }
        if bytes == b"\x1B[H" { return vec![Home] }
        if bytes == b"\x1B[F" { return vec![End] }
        if bytes == b"\x1B[Z" { return vec![ShiftTab] };

        let bs = if bytes.starts_with(BEGIN_PASTE) && bytes.ends_with(END_PASTE) {
            let start = BEGIN_PASTE.len();
            let end = bytes.len() - END_PASTE.len();
            bytes[start..end].to_vec()
        } else {
            bytes
        };

        String::from_utf8(bs)
            .unwrap()
            .chars()
            .map(Terminal::translate_char)
            .collect()
    }

    fn translate_char(c: char) -> Key {
        let numeric_char = c as u32;
        if c == '\r' {
            Enter
        } else if numeric_char == 9 {
            Tab
        } else if numeric_char == 127 {
            Backspace
        } else if numeric_char == 27 {
            Control('g')
        } else if numeric_char & 96 == 0 && numeric_char <= 128u32 {
            let c = c as u8;
            Control((c + 96u8) as char)
        } else {
            Char(c)
        }
    }

    fn write(&mut self, s: &[u8]) {
        self.output_buf.extend_from_slice(s);
    }

    fn writeln(&mut self, s: &str) {
        self.output_buf.extend_from_slice(s.as_bytes());
        self.output_buf.push(b'\n');
    }

    fn flush(&mut self) {
        self.output.write(&self.output_buf).unwrap();
        self.output_buf.clear();
    }

    fn winsize(&self) -> Option<(u16, u16)> {
        extern { fn ioctl(fd: c_int, request: c_ulong, ...) -> c_int; }
        #[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "netbsd", target_os = "openbsd"))]
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

impl Drop for Terminal {
    fn drop(&mut self) {
        self.flush();
        self.restore_tty();
    }
}

#[cfg(test)]
mod tests {
    use super::Terminal;

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
}
