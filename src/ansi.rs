use crate::screen::Key;
use crate::screen::Key::{Backspace, Char, Control, Enter, Tab};

pub fn cursor_up(lines: u16, buf: &mut [u8; 16]) -> &[u8] {
    let mut itoa_buf = itoa::Buffer::new();
    let s = itoa_buf.format(lines);
    let len = 2 + s.len() + 1;
    buf[0] = 27;
    buf[1] = b'[';
    buf[2..2 + s.len()].copy_from_slice(s.as_bytes());
    buf[2 + s.len()] = b'A';
    &buf[..len]
}

pub fn cursor_right(lines: u16, buf: &mut [u8; 16]) -> &[u8] {
    let mut itoa_buf = itoa::Buffer::new();
    let s = itoa_buf.format(lines);
    let len = 2 + s.len() + 1;
    buf[0] = 27;
    buf[1] = b'[';
    buf[2..2 + s.len()].copy_from_slice(s.as_bytes());
    buf[2 + s.len()] = b'C';
    &buf[..len]
}

pub const fn save_cursor() -> &'static [u8] {
    b"\x1b7"
}

pub const fn restore_cursor() -> &'static [u8] {
    b"\x1b8"
}

pub const fn hide_cursor() -> &'static [u8] {
    b"\x1b[?25l"
}

pub const fn show_cursor() -> &'static [u8] {
    b"\x1b[?25h"
}

pub const fn inverse() -> &'static [u8] {
    b"\x1b[7m"
}

pub const fn red() -> &'static [u8] {
    b"\x1b[31m"
}

pub const fn reset() -> &'static [u8] {
    b"\x1b[0m"
}

pub const fn blank_screen() -> &'static [u8] {
    b"\x1b[2J"
}

pub(crate) fn translate_char(c: char) -> Key {
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
