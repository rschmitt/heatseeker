#![cfg(windows)]

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
    let (cols, rows) = (80, 20);
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
}
