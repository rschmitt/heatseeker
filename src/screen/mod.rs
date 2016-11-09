#![allow(dead_code)]

#[cfg(not(windows))] pub use screen::unix::Screen;
#[cfg(windows)] pub use screen::windows::Screen;

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
}

#[cfg(not(windows))] mod unix;
#[cfg(windows)] mod windows;
