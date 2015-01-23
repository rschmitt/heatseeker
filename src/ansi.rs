const ESC: u8 = 27;

pub fn escape(sequence: &str) -> Vec<u8> {
  let mut ret = Vec::new();
  ret.push(ESC);
  ret.push_all(String::from_str("[").as_bytes());
  ret.push_all(sequence.as_bytes());
  ret
}

pub fn setpos(line: u16, column: u16) -> Vec<u8> {
  escape(format!("{};{}H", line + 1, column + 1).as_slice())
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

pub fn reset() -> Vec<u8> {
  escape("0m")
}
