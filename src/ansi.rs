const ESC: u8 = 27;

// Prepend a Control Sequence Introducer to the
// given string and return it as a Vec<u8>
fn csi(sequence: &str) -> Vec<u8> {
    let mut ret = Vec::new();
    ret.push(ESC);
    ret.push(b'[');
    ret.extend(sequence.as_bytes().iter().cloned());
    ret
}

pub fn cursor_up(lines: u16) -> Vec<u8> {
    csi(&format!("{}A", lines))
}

pub fn cursor_right(lines: u16) -> Vec<u8> {
    csi(&format!("{}C", lines))
}

pub fn save_cursor() -> Vec<u8> {
    csi("s")
}

pub fn restore_cursor() -> Vec<u8> {
    csi("u")
}

pub fn hide_cursor() -> Vec<u8> {
    csi("?25l")
}

pub fn show_cursor() -> Vec<u8> {
    csi("?25h")
}

pub fn inverse() -> Vec<u8> {
    csi("7m")
}

pub fn red() -> Vec<u8> {
    csi("31m")
}

pub fn reset() -> Vec<u8> {
    csi("0m")
}

pub fn blank_screen() -> Vec<u8> {
    csi("2J")
}
