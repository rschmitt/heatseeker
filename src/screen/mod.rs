use crate::ansi;
use std::cmp::min;
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
    #[allow(dead_code)]
    Nothing,
    Down,
    Up,
    Home,
    End,
    PgDown,
    PgUp,
    #[allow(dead_code)]
    Resize,
}

pub trait Screen {
    fn winsize(&self) -> Option<(u16, u16)>;
    fn write_bytes(&mut self, bytes: &[u8]);
    fn flush(&mut self);
    fn desired_rows(&self) -> u16;

    fn rows(&self) -> u16 {
        let (_, rows) = self.winsize().unwrap();
        rows
    }

    fn width(&self) -> u16 {
        let (cols, _) = self.winsize().unwrap();
        cols
    }

    fn visible_choices(&self) -> u16 {
        min(self.desired_rows(), self.rows().saturating_sub(1))
    }

    fn move_cursor_to_prompt_line(&mut self, col: u16) {
        self.reset_cursor();
        let mut buf = [0u8; 16];
        self.write_bytes(ansi::cursor_right(col, &mut buf));
    }

    fn reset_cursor(&mut self) {
        self.write_bytes(ansi::restore_cursor());
        self.write_bytes(b"\r");
        let num_lines = self.visible_choices();
        let mut buf = [0u8; 16];
        self.write_bytes(ansi::cursor_up(num_lines, &mut buf));
    }

    fn blank_screen(&mut self) {
        self.reset_cursor();
        let blank_line = " ".repeat(self.width() as usize);
        for _ in 0..=self.visible_choices() {
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

    fn blank_entire_screen(&mut self) {
        self.write_bytes(ansi::blank_screen());
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
    Box::from(WindowsScreen::open_screen(desired_rows))
}

#[cfg(not(windows))]
pub fn new(desired_rows: u16) -> Box<dyn Screen> {
    Box::from(UnixScreen::open_screen(desired_rows))
}

#[cfg(not(windows))]
mod unix;
#[cfg(windows)]
mod windows;
