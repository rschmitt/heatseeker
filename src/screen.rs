#![allow(dead_code)]

use self::Key::*;
use std::io::{File, Open, Read, Write};
use libc::{c_ushort, c_int, c_ulong};
use std::os::unix::AsRawFd;
use std::io::process::{Command, InheritFd};
use std::iter::repeat;
use ansi;

pub struct Screen {
  pub tty: Terminal,
  original_stty_state: Vec<u8>,
  pub height: u16,
  pub width: u16,
}

impl Screen {
  pub fn open_screen() -> Screen {
    let mut tty = Terminal::open_terminal();
    let current_stty_state = tty.stty(&["-g"]);
    tty.initialize();
    let (cols, rows) = tty.winsize().unwrap();
    Screen { tty: tty, original_stty_state: current_stty_state, height: rows, width: cols }
  }

  pub fn restore_tty(&mut self) {
    self.tty.stty(&[String::from_utf8(self.original_stty_state.clone()).unwrap().as_slice()]);
  }

  pub fn move_cursor(&mut self, line: u16, column: u16) {
    self.tty.write(ansi::setpos(line, column).as_slice());
  }

  pub fn blank_screen(&mut self, start_line: u16) {
    self.move_cursor(start_line, 0);
    let blank_line = repeat(' ').take(self.width as usize).collect::<String>();
    for _ in range(0, self.height) {
      self.tty.write(blank_line.as_bytes());
    }
  }

  pub fn show_cursor(&mut self) {
    self.tty.write(ansi::show_cursor().as_slice());
  }

  pub fn hide_cursor(&mut self) {
    self.tty.write(ansi::hide_cursor().as_slice());
  }

  pub fn write(&mut self, s: &str) {
    self.tty.write(s.as_bytes());
  }

  pub fn write_inverted(&mut self, s: &str) {
    self.tty.write(ansi::inverse().as_slice());
    self.tty.write(s.as_bytes());
    self.tty.write(ansi::reset().as_slice());
  }
}

impl Drop for Screen {
  fn drop(&mut self) {
    self.restore_tty();
  }
}

pub struct Terminal {
  input: File,
  output: File,
}

pub enum Key {
  Char(char),
  Control(char),
  Enter,
  Backspace,
}

impl Terminal {
  pub fn open_terminal() -> Terminal {
    let term_path = Path::new("/dev/tty");
    let input_file = File::open_mode(&term_path, Open, Read).unwrap();
    let output_file = File::open_mode(&term_path, Open, Write).unwrap();
    Terminal { input: input_file, output: output_file }
  }

  fn initialize(&mut self) {
    self.stty(&["raw", "-echo", "cbreak"]);
  }

  fn stty(&mut self, args: &[&str]) -> Vec<u8> {
    let term_input = File::open_mode(&Path::new("/dev/tty"), Open, Read).unwrap();
    let fd = term_input.as_raw_fd();
    let mut process = match Command::new("stty").args(args).stdin(InheritFd(fd)).spawn() {
      Ok(p) => p,
      Err(e) => panic!("Command failed: {}", e),
    };

    process.stdout.as_mut().unwrap().read_to_end().unwrap()
  }

  pub fn getchar(&mut self) -> Key {
    let byte = self.input.read_byte().unwrap();
    if byte == '\r' as u8 {
      Enter
    } else if byte == 127 {
      Backspace
    } else if byte & 96 == 0 {
      Control((byte + 96u8) as char)
    } else {
      Char(byte as char)
    }
  }

  pub fn write(&mut self, s: &[u8]) {
    self.output.write(s.as_slice()).unwrap();
  }

  pub fn writeln(&mut self, s: &str) {
    self.output.write_line(s).unwrap();
  }

  pub fn winsize(&self) -> Option<(u16, u16)> {
    extern { fn ioctl(fd: c_int, request: c_ulong, ...) -> c_int; }
    const TIOCGWINSZ: c_ulong = 0x40087468;

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
