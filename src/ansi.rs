use crate::screen::Key;
use crate::screen::Key::{
    Backspace, Char, Control, Down, End, Enter, Home, PgDown, PgUp, Resize, ShiftTab, Tab, Up,
};
use crate::{ansi, logging};

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

pub(crate) fn translate_bytes(bytes: &[u8]) -> Vec<Key> {
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
        // SIGWINCH
        (&[0x9Cu8], Some(Resize)),
    ];

    logging::log_bytes("translate_bytes", bytes);

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
            result.push(ansi::translate_char(bytes[i] as char));
            i += 1;
        }
    }

    #[cfg(debug_assertions)]
    logging::log_line(&format!("[translate_bytes] {result:?}"));
    result
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

#[cfg(test)]
mod tests {
    use super::Key::*;
    use super::translate_bytes;

    #[test]
    fn translate_bytes_escape() {
        assert_eq!(translate_bytes(&[27u8]), vec![Control('g')]);
    }

    #[test]
    fn translate_bytes_down_arrow() {
        assert_eq!(translate_bytes(b"\x1B[A"), vec![Up]);
    }

    #[test]
    fn translate_bytes_down_arrows() {
        assert_eq!(translate_bytes(b"\x1B[A\x1B[A"), vec![Up, Up]);
    }

    #[test]
    fn translate_bytes_mixed() {
        assert_eq!(translate_bytes(b"\x1BOAa\x1BOA"), vec![Up, Char('a'), Up]);
        assert_eq!(translate_bytes(b"\x1B[Aa\x1B[A"), vec![Up, Char('a'), Up]);
        assert_eq!(translate_bytes(b"\x1BOAa\x1B[A"), vec![Up, Char('a'), Up]);
        assert_eq!(translate_bytes(b"\x1B[Aa\x1BOA"), vec![Up, Char('a'), Up]);
        assert_eq!(translate_bytes(b"a\x1BOAb"), vec![Char('a'), Up, Char('b')]);
        assert_eq!(translate_bytes(b"ab\x1BOA"), vec![Char('a'), Char('b'), Up]);
    }

    #[test]
    fn translate_bytes_chars() {
        assert_eq!(translate_bytes(b"Ab"), vec![Char('A'), Char('b')]);
    }

    #[test]
    fn translate_bytes_paste() {
        const BEGIN_PASTE: &[u8] = b"\x1B[200~";
        const END_PASTE: &[u8] = b"\x1B[201~";

        let input = [BEGIN_PASTE, b"a", END_PASTE].concat();
        assert_eq!(translate_bytes(&input), vec![Char('a')]);

        let input = [b"a", BEGIN_PASTE, b"b", END_PASTE, b"c"].concat();
        assert_eq!(
            translate_bytes(&input),
            vec![Char('a'), Char('b'), Char('c')]
        );

        assert_eq!(translate_bytes(BEGIN_PASTE), vec![]);
        assert_eq!(translate_bytes(END_PASTE), vec![]);

        let input = [b"a", BEGIN_PASTE, b"b"].concat();
        assert_eq!(translate_bytes(&input), vec![Char('a'), Char('b')]);
    }
}
