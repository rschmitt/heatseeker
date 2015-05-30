const ESC: u8 = 27;

pub fn escape(sequence: &str) -> Vec<u8> {
    let mut ret = Vec::new();
    ret.push(ESC);
    ret.extend("[".to_string().as_bytes().iter().map(|&i| i));
    ret.extend(sequence.as_bytes().iter().map(|&i| i));
    ret
}

pub fn setpos(line: u16, column: u16) -> Vec<u8> {
    escape(&format!("{};{}H", line + 1, column + 1))
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
