#![allow(dead_code)]

use crate::ansi;
#[cfg(not(windows))]
use unix::UnixScreen;
#[cfg(windows)]
pub use windows::WindowsScreen;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Key {
    Char(char),
    Control(char),
    Enter,
    Backspace,
    Tab,
    ShiftTab,
    Nothing,
    Down,
    Up,
    Home,
    End,
    PgDown,
    PgUp,
}

pub trait Screen {
    fn visible_choices(&self) -> u16;
    fn width(&self) -> u16;
    fn reset_cursor(&mut self);
    fn write_bytes(&mut self, bytes: &[u8]);
    fn flush(&mut self);

    fn move_cursor_to_prompt_line(&mut self, col: u16) {
        self.reset_cursor();
        let mut buf = [0u8; 16];
        self.write_bytes(ansi::cursor_right(col, &mut buf));
    }

    fn blank_screen(&mut self) {
        self.reset_cursor();
        let blank_line = " ".repeat(self.width() as usize);
        for _ in 0..self.visible_choices() + 1 {
            self.write_bytes(blank_line.as_bytes());
        }
        self.reset_cursor();
    }

    fn show_cursor(&mut self) {
        self.write_bytes(ansi::show_cursor());
        self.flush();
    }

    fn hide_cursor(&mut self) {
        self.write_bytes(ansi::hide_cursor());
    }

    fn write(&mut self, s: &str) {
        self.write_bytes(s.as_bytes());
    }

    fn write_red_inverted(&mut self, s: &str) {
        self.write_bytes(ansi::red());
        self.write_bytes(ansi::inverse());
        self.write(s);
        self.write_bytes(ansi::reset());
    }

    fn write_red(&mut self, s: &str) {
        self.write_bytes(ansi::red());
        self.write(s);
        self.write_bytes(ansi::reset());
    }

    fn write_inverted(&mut self, s: &str) {
        self.write_bytes(ansi::inverse());
        self.write(s);
        self.write_bytes(ansi::reset());
    }

    fn get_buffered_keys(&mut self) -> Vec<Key>;
}

#[cfg(windows)]
pub fn new(desired_rows: u16) -> Box<dyn Screen> {
    if WindowsScreen::is_cygwin() {
        //        UnixScreen::open_screen(desired_rows)
        panic!("This executable does not support Cygwin.");
    } else {
        Box::from(WindowsScreen::open_screen(desired_rows))
    }
}

#[cfg(not(windows))]
pub fn new(desired_rows: u16) -> Box<dyn Screen> {
    Box::from(UnixScreen::open_screen(desired_rows))
}

#[cfg(not(windows))]
mod unix;
#[cfg(windows)]
mod windows;
