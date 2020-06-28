#![allow(dead_code)]

#[cfg(not(windows))] use unix::UnixScreen;
#[cfg(windows)] pub use windows::WindowsScreen;

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
    fn move_cursor_to_prompt_line(&mut self, col: u16);
    fn blank_screen(&mut self);
    fn show_cursor(&mut self);
    fn hide_cursor(&mut self);
    fn write(&mut self, s: &str);
    fn write_red_inverted(&mut self, s: &str);
    fn write_red(&mut self, s: &str);
    fn write_inverted(&mut self, s: &str);
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

#[cfg(not(windows))] mod unix;
#[cfg(windows)] mod windows;
