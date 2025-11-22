#![cfg(not(windows))]

use super::Key;
use super::Key::*;
use crate::NEWLINE;
use crate::ansi;
use std::cmp::min;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::io::*;
use std::path::*;
use std::process::Command;
use std::process::Stdio;
use std::str;

use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread;

use crate::screen::Screen;
use ::libc::{SIGINT, SIGWINCH, c_int, c_ulong, c_ushort, dup};

pub struct UnixScreen {
    tty: Terminal,
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

    fn reset_cursor(&mut self) {
        self.tty.write(ansi::restore_cursor());

        let num_lines = self.visible_choices();
        let mut buf = [0u8; 16];
        self.tty.write(ansi::cursor_up(num_lines, &mut buf));
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        self.tty.write(bytes);
    }

    fn flush(&mut self) {
        self.tty.flush();
    }

    // Return all buffered keystrokes, or the next key if buffer is empty.
    fn get_buffered_keys(&mut self) -> Vec<Key> {
        let mut ret = Vec::new();
        while let Ok(bytes) = self.tty.input.try_recv() {
            ret.extend(bytes);
        }
        while ret.is_empty() {
            let bytes = self.tty.input.recv().unwrap();
            if bytes == vec![128 + SIGWINCH as u8] {
                self.blank_entire_screen();
                return vec![Nothing];
            } else if bytes == vec![128 + SIGINT as u8] {
                return vec![Control('g')];
            } else {
                ret.extend(bytes);
            }
        }
        Terminal::translate_bytes(ret.clone())
    }
}

impl UnixScreen {
    pub fn open_screen(desired_rows: u16) -> UnixScreen {
        let mut tty = Terminal::open_terminal();
        tty.write(ansi::reset());
        let (_, rows) = tty.winsize().unwrap();
        let visible_choices = min(desired_rows, rows - 1);
        for _ in 0..visible_choices {
            tty.write(NEWLINE.as_bytes());
        }
        tty.write(ansi::save_cursor());

        UnixScreen { tty, desired_rows }
    }

    pub fn blank_entire_screen(&mut self) {
        self.tty.write(ansi::blank_screen());
    }
}

struct Terminal {
    input: Receiver<Vec<u8>>,
    input_fd: RawFd,
    output: File,
    output_buf: Vec<u8>,
    original_stty_state: Vec<u8>,
}

fn start_sigwinch_handler(tx: Sender<Vec<u8>>) {
    let mut signals = signal_hook::iterator::Signals::new([SIGWINCH, SIGINT]).unwrap();
    thread::spawn(move || {
        for signal in signals.forever() {
            tx.send(vec![128 + signal as u8]).unwrap();
        }
    });
}

impl Terminal {
    fn open_terminal() -> Terminal {
        let term_path = Path::new("/dev/tty");
        let mut input_file = File::open(term_path).unwrap();
        let output_file = OpenOptions::new().write(true).open(term_path).unwrap();
        let input_fd = input_file.as_raw_fd();
        let (tx, rx) = mpsc::channel();

        start_sigwinch_handler(tx.clone());

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
            process
                .stdout
                .as_mut()
                .unwrap()
                .read_to_end(&mut buf)
                .unwrap();
            let mut str = String::from_utf8(buf).unwrap();

            // The output from `stty -g` may include a newline, which we have to strip off. Otherwise,
            // when we go to restore the tty, stty (on some platforms) will fail with an "invalid
            // argument" error.
            crate::trim(&mut str);

            str.into_bytes()
        } else {
            process
                .stderr
                .as_mut()
                .unwrap()
                .read_to_end(&mut buf)
                .unwrap();
            panic!(
                "stty invocation failed: {}",
                String::from_utf8(buf).unwrap()
            );
        }
    }

    fn translate_bytes(bytes: Vec<u8>) -> Vec<Key> {
        const SEQUENCES: &[(&[u8], Option<Key>)] = &[
            (b"\x1B[5~", Some(PgUp)),
            (b"\x1B[6~", Some(PgDown)),
            (b"\x1B[H", Some(Home)),
            (b"\x1B[F", Some(End)),
            (b"\x1B[Z", Some(ShiftTab)),
            // Arrow keys
            (b"\x1B[A", Some(Up)),
            (b"\x1BOA", Some(Up)),
            (b"\x1B[B", Some(Down)),
            (b"\x1BOB", Some(Down)),
            (b"\x1B[C", None),
            (b"\x1BOC", None),
            (b"\x1B[D", None),
            (b"\x1BOD", None),
            // Paste markers
            (b"\x1B[200~", None),
            (b"\x1B[201~", None),
        ];

        let mut result = Vec::new();
        let mut i = 0;

        while i < bytes.len() {
            let current = &bytes[i..];
            let mut matched = false;

            for &(seq, key) in SEQUENCES {
                if current.starts_with(seq) {
                    if let Some(k) = key {
                        result.push(k);
                    }
                    i += seq.len();
                    matched = true;
                    break;
                }
            }

            if !matched {
                result.push(Terminal::translate_char(bytes[i] as char));
                i += 1;
            }
        }

        result
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

    fn flush(&mut self) {
        self.output.write_all(&self.output_buf).unwrap();
        self.output_buf.clear();
    }

    fn winsize(&self) -> Option<(u16, u16)> {
        unsafe extern "C" {
            fn ioctl(fd: c_int, request: c_ulong, ...) -> c_int;
        }
        #[cfg(any(
            target_os = "macos",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
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

        let mut size = TermSize {
            rows: 0,
            cols: 0,
            x: 0,
            y: 0,
        };
        if unsafe { ioctl(self.output.as_raw_fd(), TIOCGWINSZ, &mut size) } == 0 {
            Some((size.cols, size.rows))
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
    use super::Key::*;
    use super::Terminal;
    use libc::{STDIN_FILENO, STDOUT_FILENO, isatty};

    #[test]
    fn winsize_test() {
        let has_tty = unsafe { isatty(STDIN_FILENO) != 0 || isatty(STDOUT_FILENO) != 0 };
        if !has_tty {
            // Skip when no interactive tty is available (e.g., CI sandboxes).
            return;
        }
        let term = Terminal::open_terminal();
        let (cols, rows) = term.winsize().expect("Failed to get window size!");
        // We don't know the window size a priori, but we can at least
        // assert that it is within some kind of sensible range.
        assert!(cols > 40);
        assert!(rows > 10);
        assert!(cols < 1000);
        assert!(rows < 1000);
    }

    #[test]
    fn translate_bytes_escape() {
        assert_eq!(Terminal::translate_bytes(vec![27u8]), vec![Control('g')]);
    }

    #[test]
    fn translate_bytes_down_arrow() {
        assert_eq!(Terminal::translate_bytes(b"\x1B[A".to_vec()), vec![Up]);
    }

    #[test]
    fn translate_bytes_down_arrows() {
        assert_eq!(
            Terminal::translate_bytes(b"\x1B[A\x1B[A".to_vec()),
            vec![Up, Up]
        );
    }

    #[test]
    fn translate_bytes_mixed() {
        assert_eq!(
            Terminal::translate_bytes(b"\x1BOAa\x1BOA".to_vec()),
            vec![Up, Char('a'), Up]
        );
        assert_eq!(
            Terminal::translate_bytes(b"\x1B[Aa\x1B[A".to_vec()),
            vec![Up, Char('a'), Up]
        );
        assert_eq!(
            Terminal::translate_bytes(b"\x1BOAa\x1B[A".to_vec()),
            vec![Up, Char('a'), Up]
        );
        assert_eq!(
            Terminal::translate_bytes(b"\x1B[Aa\x1BOA".to_vec()),
            vec![Up, Char('a'), Up]
        );
        assert_eq!(
            Terminal::translate_bytes(b"a\x1BOAb".to_vec()),
            vec![Char('a'), Up, Char('b')]
        );
        assert_eq!(
            Terminal::translate_bytes(b"ab\x1BOA".to_vec()),
            vec![Char('a'), Char('b'), Up]
        );
    }

    #[test]
    fn translate_bytes_chars() {
        assert_eq!(
            Terminal::translate_bytes(b"Ab".to_vec()),
            vec![Char('A'), Char('b')]
        );
    }

    #[test]
    fn translate_bytes_paste() {
        const BEGIN_PASTE: &[u8] = b"\x1B[200~";
        const END_PASTE: &[u8] = b"\x1B[201~";

        let input = [BEGIN_PASTE, b"a", END_PASTE].concat();
        assert_eq!(Terminal::translate_bytes(input), vec![Char('a')]);

        let input = [b"a", BEGIN_PASTE, b"b", END_PASTE, b"c"].concat();
        assert_eq!(
            Terminal::translate_bytes(input),
            vec![Char('a'), Char('b'), Char('c')]
        );

        assert_eq!(Terminal::translate_bytes(BEGIN_PASTE.to_vec()), vec![]);
        assert_eq!(Terminal::translate_bytes(END_PASTE.to_vec()), vec![]);

        let input = [b"a", BEGIN_PASTE, b"b"].concat();
        assert_eq!(Terminal::translate_bytes(input), vec![Char('a'), Char('b')]);
    }
}
