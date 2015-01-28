#![cfg(windows)]
#![allow(unused_variables)]

extern crate libc;

use libc::*;
use std::cmp::min;
use screen::Key;
use screen::Key::*;

pub struct Screen {
  pub height: u16,
  pub width: u16,
  pub visible_choices: u16,
  pub start_line: u16,
}

impl Screen {
  pub fn open_screen() -> Screen {
    let (cols, rows) = Screen::winsize().unwrap();
    let visible_choices = min(20, rows - 1);
    let start_line = rows - visible_choices - 1;
    Screen {
      height: rows,
      width: cols,
      visible_choices: visible_choices,
      start_line: start_line,
    }
  }

  pub fn move_cursor(&mut self, line: u16, column: u16) {
  }

  pub fn move_cursor_to_bottom(&mut self) {
  }

  pub fn blank_screen(&mut self) {
  }

  pub fn show_cursor(&mut self) {
  }

  pub fn hide_cursor(&mut self) {
  }

  pub fn write(&mut self, s: &str) {
  }

  pub fn write_inverted(&mut self, s: &str) {
  }

  // Return all buffered keystrokes, or the next key if buffer is empty.
  pub fn get_buffered_keys(&mut self) -> Vec<Key> {
    let mut ret = Vec::new();
    ret.push(Enter);
    ret
  }

  fn winsize() -> Option<(u16, u16)> {
    #[allow(non_snake_case)]
    #[repr(C)]
    struct CONSOLE_SCREEN_BUFFER_INFO {
      dwSize: [c_short; 2],
      dwCursorPosition: [c_short; 2],
      wAttributes: WORD,
      srWindow: [c_short; 4],
      dwMaximumWindowSize: [c_short; 2],
    }

    extern {
      fn GetStdHandle(nStdHandle: DWORD) -> HANDLE;
      fn GetConsoleScreenBufferInfo(
        hConsoleOutput: HANDLE,
        lpConsoleScreenBufferInfo: *mut CONSOLE_SCREEN_BUFFER_INFO
      ) -> BOOL;
    }

    let mut buffer_info;
    let result;
    unsafe {
      buffer_info = ::std::mem::uninitialized();
      result = GetConsoleScreenBufferInfo(GetStdHandle(-11), &mut buffer_info);
    }
    if result != 0 {
      // This code specifically computes the size of the window,
      // *not* the size of the buffer (which is easily available
      // from dwSize). I got the algorithm from:
      //
      // http://stackoverflow.com/a/12642749
      let left = buffer_info.srWindow[0];
      let top = buffer_info.srWindow[1];
      let right = buffer_info.srWindow[2];
      let bottom = buffer_info.srWindow[3];
      let cols = right - left + 1;
      let rows = bottom - top + 1;
      Some((cols as u16, rows as u16))
    } else {
      None
    }
  }
}

#[test]
fn winsize_test() {
  let (cols, rows) = Screen::winsize().expect("Failed to get window size!");
  // We don't know the window size a priori, but we can at least
  // assert that it is within some kind of sensible range.
  assert!(cols > 40);
  assert!(rows > 40);
  assert!(cols < 1000);
  assert!(rows < 1000);
}
