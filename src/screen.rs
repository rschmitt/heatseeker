#![allow(dead_code)]

use self::Key::*;
use std::io::{File, Open, Read, Write};
use libc::{c_ushort, c_int, c_ulong};
use std::os::unix::AsRawFd;
use std::io::process::{Command, InheritFd};
use std::iter::repeat;
use std::cmp::min;
use ansi;

use std::thread::Thread;
use std::sync::mpsc::{Sender, Receiver};
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

  pub fn restore_tty(&mut self) {
    self.tty.stty(&[String::from_utf8(self.original_stty_state.clone()).unwrap().as_slice()]);
  }

  pub fn move_cursor(&mut self, line: u16, column: u16) {
    self.tty.write(ansi::setpos(line, column).as_slice());
  }

  pub fn blank_screen(&mut self) {
    let start_line = self.start_line;
    self.move_cursor(start_line, 0);
    let blank_line = repeat(' ').take(self.width as usize).collect::<String>();
    for _ in range(0, self.height) {
      self.tty.write(blank_line.as_bytes());
    }
    self.move_cursor(start_line, 0);
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

  // Return all buffered keystrokes, or the next key if buffer is empty.
  pub fn get_buffered_keys(&mut self) -> Vec<Key> {
    let mut ret = Vec::new();
    while let Ok(byte) = self.tty.input.try_recv() {
      ret.push(Terminal::translate_byte(byte));
    }
    if ret.is_empty() {
      let byte = self.tty.input.recv().unwrap();
      ret.push(Terminal::translate_byte(byte));
    }
    ret
  }
}

impl Drop for Screen {
  fn drop(&mut self) {
    self.restore_tty();
  }
}

pub enum Key {
  Char(char),
  Control(char),
  Enter,
  Backspace,
}

struct Terminal {
  input: Receiver<u8>,
  output: File,
}

impl Terminal {
  fn open_terminal() -> Terminal {
    let term_path = Path::new("/dev/tty");
    let mut input_file = File::open_mode(&term_path, Open, Read).unwrap();
    let output_file = File::open_mode(&term_path, Open, Write).unwrap();
    let (tx, rx): (Sender<u8>, Receiver<u8>) = mpsc::channel();
    Thread::spawn(move || {
      loop {
        tx.send(input_file.read_byte().unwrap()).unwrap();
      }
    });
    Terminal {
      input: rx,
      output: output_file
    }
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

  fn getchar(&mut self) -> Key {
    let byte = self.input.recv().unwrap();
    Terminal::translate_byte(byte)
  }

  fn translate_byte(byte: u8) -> Key {
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

  fn write(&mut self, s: &[u8]) {
    self.output.write(s.as_slice()).unwrap();
  }

  fn writeln(&mut self, s: &str) {
    self.output.write_line(s).unwrap();
  }

  fn winsize(&self) -> Option<(u16, u16)> {
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
