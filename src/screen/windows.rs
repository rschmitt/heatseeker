use std::cmp::min;
use std::ffi::OsString;
use std::mem::size_of;
use std::os::windows::ffi::OsStringExt;
use std::slice::from_raw_parts;

use windows::Win32::Foundation::{HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_GENERIC_READ, FILE_GENERIC_WRITE, FILE_NAME_INFO,
    FILE_SHARE_READ, FILE_SHARE_WRITE, FileNameInfo, GetFileInformationByHandleEx, OPEN_EXISTING,
};
use windows::Win32::System::Console::{
    BACKGROUND_BLUE, BACKGROUND_GREEN, BACKGROUND_RED, CONSOLE_CHARACTER_ATTRIBUTES,
    CONSOLE_CURSOR_INFO, CONSOLE_MODE, CONSOLE_SCREEN_BUFFER_INFO, COORD, ENABLE_ECHO_INPUT,
    ENABLE_LINE_INPUT, ENABLE_PROCESSED_INPUT, FOREGROUND_RED, GetConsoleCursorInfo,
    GetConsoleMode, GetConsoleScreenBufferInfo, INPUT_RECORD, KEY_EVENT, ReadConsoleInputW,
    SetConsoleCursorInfo, SetConsoleCursorPosition, SetConsoleMode, SetConsoleTextAttribute,
    WriteConsoleW,
};
use windows::Win32::System::Console::{GetStdHandle, STD_INPUT_HANDLE};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    VK_BACK, VK_DOWN, VK_END, VK_ESCAPE, VK_HOME, VK_NEXT, VK_PRIOR, VK_RETURN, VK_SHIFT, VK_TAB,
    VK_UP,
};
use windows::core::w;

use super::Key;
use super::Key::*;
use super::Screen;
use crate::NEWLINE;

macro_rules! win32 {
    ($expr:expr) => {{
        let result = unsafe { $expr };
        if result.is_err() {
            panic!("win32 call failed: {}", std::io::Error::last_os_error());
        }
    }};
}

pub struct WindowsScreen {
    visible_choices: u16,
    start_line: u16,
    original_console_mode: CONSOLE_MODE,
    original_colors: CONSOLE_CHARACTER_ATTRIBUTES,
    conin: HANDLE,
    conout: HANDLE,
    default_cursor_info: CONSOLE_CURSOR_INFO,
    shifted: bool,
}

impl Screen for WindowsScreen {
    fn visible_choices(&self) -> u16 {
        self.visible_choices
    }

    fn width(&self) -> u16 {
        let (cols, _) = WindowsScreen::winsize(self.conout).unwrap();
        cols
    }

    fn move_cursor_to_prompt_line(&mut self, col: u16) {
        let start_line = self.start_line;
        self.move_cursor(start_line, col);
    }

    fn blank_screen(&mut self) {
        let blank_line = " ".repeat((self.width() - 1) as usize);
        let start_line = self.start_line;
        self.move_cursor(start_line, 0);
        for _ in 0..self.visible_choices {
            self.write(&blank_line);
            self.write(NEWLINE);
        }
        self.write(&blank_line);
        self.move_cursor(start_line, 0);
    }

    fn show_cursor(&mut self) {
        win32!(SetConsoleCursorInfo(self.conout, &self.default_cursor_info));
    }

    fn hide_cursor(&mut self) {
        let cursor_info = CONSOLE_CURSOR_INFO {
            dwSize: 100,
            bVisible: false.into(),
        };
        win32!(SetConsoleCursorInfo(self.conout, &cursor_info));
    }

    fn write(&mut self, s: &str) {
        Self::write_to(self.conout, s);
    }

    fn write_red_inverted(&mut self, s: &str) {
        let orig = self.original_colors;
        const WHITE_ON_RED: CONSOLE_CHARACTER_ATTRIBUTES = BACKGROUND_RED;
        self.set_colors(WHITE_ON_RED);
        self.write(s);
        self.set_colors(orig);
    }

    fn write_red(&mut self, s: &str) {
        let orig = self.original_colors;
        const RED_ON_BLACK: CONSOLE_CHARACTER_ATTRIBUTES = FOREGROUND_RED;
        self.set_colors(RED_ON_BLACK);
        self.write(s);
        self.set_colors(orig);
    }

    fn write_inverted(&mut self, s: &str) {
        let orig = self.original_colors;
        let black_on_white: CONSOLE_CHARACTER_ATTRIBUTES =
            BACKGROUND_RED | BACKGROUND_GREEN | BACKGROUND_BLUE;
        self.set_colors(black_on_white);
        self.write(s);
        self.set_colors(orig);
    }

    fn get_buffered_keys(&mut self) -> Vec<Key> {
        let mut ret = Vec::new();
        let mut input_record = [INPUT_RECORD::default(); 100];
        let mut events_read: u32 = 0;
        win32!(ReadConsoleInputW(
            self.conin,
            &mut input_record,
            &mut events_read
        ));
        for rec in input_record {
            ret.push(WindowsScreen::translate_event(rec, &mut self.shifted));
        }
        ret
    }
}

impl Drop for WindowsScreen {
    fn drop(&mut self) {
        win32!(SetConsoleMode(self.conin, self.original_console_mode));
    }
}

impl WindowsScreen {
    pub fn is_cygwin() -> bool {
        let size = size_of::<FILE_NAME_INFO>();
        let mut name_info_bytes = vec![0u8; size + windows::Win32::Foundation::MAX_PATH as usize];
        let stdin: HANDLE = unsafe { GetStdHandle(STD_INPUT_HANDLE).unwrap() };
        let ok = unsafe {
            GetFileInformationByHandleEx(
                stdin,
                FileNameInfo,
                name_info_bytes.as_mut_ptr() as *mut _,
                name_info_bytes.len() as u32,
            )
        };
        if ok.is_err() {
            // on native windows this typically fails for interactive stdin; treat as not cygwin/msys
            false
        } else {
            // interpret FILE_NAME_INFO { FileNameLength, FileName[...] }
            let file_name_len_bytes =
                unsafe { *(name_info_bytes.as_ptr() as *const FILE_NAME_INFO) }.FileNameLength
                    as usize;
            let name_bytes = &name_info_bytes[size..size + file_name_len_bytes];
            let name_u16 =
                unsafe { from_raw_parts(name_bytes.as_ptr() as *const u16, name_bytes.len() / 2) };
            let name = OsString::from_wide(name_u16)
                .as_os_str()
                .to_string_lossy()
                .into_owned();
            name.contains("msys-") || name.contains("-pty") || name.contains("cygwin-")
        }
    }

    pub fn open_screen(desired_rows: u16) -> WindowsScreen {
        let mut orig_mode: CONSOLE_MODE = Default::default();
        let conin: HANDLE;
        let conout: HANDLE;

        unsafe {
            let rw_access = FILE_GENERIC_READ | FILE_GENERIC_WRITE;
            conin = CreateFileW(
                w!("CONIN$"),
                rw_access.0,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                None,
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                None,
            )
            .unwrap();
            conout = CreateFileW(
                w!("CONOUT$"),
                rw_access.0,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                None,
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                None,
            )
            .unwrap();
        }

        if conin == INVALID_HANDLE_VALUE || conout == INVALID_HANDLE_VALUE {
            panic!("Unable to open console");
        }

        let (_, rows) = WindowsScreen::winsize(conout).unwrap();

        win32!(GetConsoleMode(conin, &mut orig_mode));
        let new_mode =
            orig_mode & !(ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT | ENABLE_PROCESSED_INPUT);
        win32!(SetConsoleMode(conin, new_mode));

        let mut default_cursor_info = CONSOLE_CURSOR_INFO {
            dwSize: 100,
            bVisible: true.into(),
        };
        win32!(GetConsoleCursorInfo(conout, &mut default_cursor_info));

        let initial_pos = WindowsScreen::get_cursor_pos(conout);
        let visible_choices = min(desired_rows, rows - 1);
        let start_line = get_start_line(rows, visible_choices, initial_pos);
        let original_colors = WindowsScreen::get_original_colors(conout);
        let (column, _) = initial_pos;
        if column > 0 {
            Self::write_to(conout, NEWLINE);
        }
        for _ in 0..visible_choices {
            Self::write_to(conout, NEWLINE);
        }
        WindowsScreen {
            visible_choices,
            start_line: start_line + Self::get_buffer_offset(conout),
            original_console_mode: orig_mode,
            original_colors,
            conin,
            conout,
            default_cursor_info,
            shifted: false,
        }
    }

    fn write_to(conout: HANDLE, s: &str) {
        let utf16: Vec<u16> = s.encode_utf16().collect();
        let mut chars_written: u32 = 0;
        win32!(WriteConsoleW(
            conout,
            &utf16,
            Some(&mut chars_written),
            None
        ));
    }

    fn move_cursor(&mut self, line: u16, column: u16) {
        win32!(SetConsoleCursorPosition(
            self.conout,
            COORD {
                X: column as i16,
                Y: line as i16
            }
        ));
    }

    fn set_colors(&mut self, colors: CONSOLE_CHARACTER_ATTRIBUTES) {
        win32!(SetConsoleTextAttribute(self.conout, colors));
    }

    fn translate_event(event: INPUT_RECORD, shifted: &mut bool) -> Key {
        if u32::from(event.EventType) != KEY_EVENT {
            return Nothing;
        }

        let key_event = unsafe { event.Event.KeyEvent };
        let vk_code = key_event.wVirtualKeyCode;

        if vk_code == VK_SHIFT.0 {
            *shifted = key_event.bKeyDown.as_bool();
            return Nothing;
        }

        if !key_event.bKeyDown.as_bool() {
            return Nothing;
        }

        if vk_code == VK_UP.0 {
            Up
        } else if vk_code == VK_DOWN.0 {
            Down
        } else if vk_code == VK_PRIOR.0 {
            PgUp
        } else if vk_code == VK_NEXT.0 {
            PgDown
        } else if vk_code == VK_HOME.0 {
            Home
        } else if vk_code == VK_END.0 {
            End
        } else if vk_code == VK_TAB.0 {
            if *shifted { ShiftTab } else { Tab }
        } else if vk_code == VK_BACK.0 {
            Backspace
        } else if vk_code == VK_RETURN.0 {
            Enter
        } else if vk_code == VK_ESCAPE.0 {
            Control('g')
        } else {
            let ch = unsafe { key_event.uChar.UnicodeChar };
            if ch & 96 == 0 {
                Control(((ch + 96u16) as u8) as char)
            } else {
                Char((ch as u8) as char)
            }
        }
    }

    fn get_cursor_pos(handle: HANDLE) -> (u16, u16) {
        let mut buffer_info = CONSOLE_SCREEN_BUFFER_INFO::default();
        win32!(GetConsoleScreenBufferInfo(handle, &mut buffer_info));
        let cursor_pos = buffer_info.dwCursorPosition;
        (cursor_pos.X as u16, cursor_pos.Y as u16)
    }

    fn get_original_colors(handle: HANDLE) -> CONSOLE_CHARACTER_ATTRIBUTES {
        let mut buffer_info = CONSOLE_SCREEN_BUFFER_INFO::default();
        win32!(GetConsoleScreenBufferInfo(handle, &mut buffer_info));
        buffer_info.wAttributes
    }

    fn winsize(conout: HANDLE) -> Option<(u16, u16)> {
        let mut buffer_info = CONSOLE_SCREEN_BUFFER_INFO::default();
        let result = unsafe { GetConsoleScreenBufferInfo(conout, &mut buffer_info) };
        if result.is_ok() {
            // This code specifically computes the size of the window,
            // *not* the size of the buffer (which is easily available
            // from dwSize). I got the algorithm from:
            //
            // http://stackoverflow.com/a/12642749
            let left = buffer_info.srWindow.Left;
            let top = buffer_info.srWindow.Top;
            let right = buffer_info.srWindow.Right;
            let bottom = buffer_info.srWindow.Bottom;
            let cols = right - left + 1;
            let rows = bottom - top + 1;
            Some((cols as u16, rows as u16))
        } else {
            None
        }
    }

    fn get_buffer_offset(conout: HANDLE) -> u16 {
        let mut buffer_info = CONSOLE_SCREEN_BUFFER_INFO::default();
        win32!(GetConsoleScreenBufferInfo(conout, &mut buffer_info));
        buffer_info.srWindow.Top as u16
    }
}

fn get_start_line(rows: u16, visible_choices: u16, initial_pos: (u16, u16)) -> u16 {
    let bottom_most_line = rows - visible_choices - 1;
    let (initial_x, initial_y) = initial_pos;
    let line_under_cursor = if initial_x == 0 {
        initial_y
    } else {
        initial_y + 1
    };
    if line_under_cursor + 1 + visible_choices > rows {
        bottom_most_line
    } else {
        line_under_cursor
    }
}

#[cfg(test)]
mod tests {
    use super::{WindowsScreen, get_start_line};
    use windows::Win32::System::Console::{GetStdHandle, STD_OUTPUT_HANDLE};

    #[test]
    fn winsize_test() {
        // Skip on ci without a console
        if option_env!("APPVEYOR").is_some() || option_env!("TRAVIS").is_some() {
            return;
        }
        let conout = unsafe { GetStdHandle(STD_OUTPUT_HANDLE).unwrap() };
        let (cols, rows) = WindowsScreen::winsize(conout).expect("failed to get window size");
        assert!(cols > 40 && cols < 1000);
        assert!(rows > 10 && rows < 1000);
    }

    #[test]
    fn start_line_test() {
        assert_eq!(5, get_start_line(100, 20, (0, 5)));
        assert_eq!(6, get_start_line(100, 20, (1, 5)));
        assert_eq!(79, get_start_line(100, 20, (0, 100)));
        assert_eq!(0, get_start_line(15, 14, (0, 5)));
        assert_eq!(79, get_start_line(100, 20, (50, 100)));
    }
}
