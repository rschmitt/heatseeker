const ESC: u8 = 27;

pub fn escape(sequence: &str) -> Vec<u8> {
    let mut ret = Vec::new();
    ret.push(ESC);
    ret.extend(b"[".iter().cloned());
    ret.extend(sequence.as_bytes().iter().cloned());
    ret
}

pub fn cursor_up(lines: u16) -> Vec<u8> {
    escape(&format!("{}A", lines))
}

pub fn cursor_right(lines: u16) -> Vec<u8> {
    escape(&format!("{}C", lines))
}

pub fn save_cursor() -> Vec<u8> {
    escape("s")
}

pub fn restore_cursor() -> Vec<u8> {
    escape("u")
}

pub fn hide_cursor() -> Vec<u8> {
    escape("?25l")
}

pub fn show_cursor() -> Vec<u8> {
    escape("?25h")
}

pub fn inverse() -> Vec<u8> {
    escape("7m")
}

pub fn red() -> Vec<u8> {
    escape("31m")
}

pub fn reset() -> Vec<u8> {
    escape("0m")
}
