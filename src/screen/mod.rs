#![allow(dead_code)]

#[cfg(not(windows))] pub use screen::unix::Screen;
#[cfg(windows)] pub use screen::windows::Screen;

pub enum Key {
    Char(char),
    Control(char),
    Enter,
    Backspace,
    Tab,
}

#[cfg(not(windows))] mod unix;
#[cfg(windows)] mod windows;

fn get_start_line(rows: u16, visible_choices: u16, initial_pos: (u16, u16)) -> u16 {
    let bottom_most_line = rows - visible_choices - 1;
    let (initial_x, initial_y) = initial_pos;
    let line_under_cursor = if initial_x == 0 { initial_y } else { initial_y + 1 };
    if line_under_cursor + 1 + visible_choices > rows {
        bottom_most_line
    } else {
        line_under_cursor
    }
}

#[test]
fn start_line_test() {
    assert_eq!(5, get_start_line(100, 20, (0, 5)));
    assert_eq!(6, get_start_line(100, 20, (1, 5)));
    assert_eq!(79, get_start_line(100, 20, (0, 100)));
    assert_eq!(0, get_start_line(15, 14, ((0, 5))));
    assert_eq!(79, get_start_line(100, 20, (50, 100)));
}
