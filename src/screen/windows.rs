#![cfg(windows)]

use winapi::ctypes::{c_void};
use winapi::shared::minwindef::{TRUE, FALSE, WORD, DWORD, LPDWORD, MAX_PATH};
use winapi::shared::ntdef::{HANDLE, PVOID};
use winapi::um::consoleapi::{GetConsoleMode, SetConsoleMode, ReadConsoleInputW, WriteConsoleW};
use winapi::um::fileapi::{FILE_NAME_INFO, CreateFileA};
use winapi::um::handleapi::INVALID_HANDLE_VALUE;
use winapi::um::minwinbase::FileNameInfo;
use winapi::um::processenv::GetStdHandle;
use winapi::um::winbase::{STD_INPUT_HANDLE, GetFileInformationByHandleEx};
use winapi::um::wincon::*;
use winapi::um::wincontypes::{INPUT_RECORD, PINPUT_RECORD, COORD, KEY_EVENT};
use winapi::um::winnt::{GENERIC_READ, GENERIC_WRITE, FILE_SHARE_READ};
use std::ffi::OsString;
use std::io;
use std::os::windows::ffi::OsStringExt;
use std::ptr;
use std::iter::repeat;
use std::cmp::min;
use super::Key;
use super::Key::*;
use super::Screen;

use std::thread;
use std::slice::from_raw_parts;
use std::sync::mpsc::Receiver;
use std::sync::mpsc;
use crate::NEWLINE;

macro_rules! win32 {
    ($funcall:expr) => (
        if unsafe { $funcall } == 0 {
            panic!("Win32 call failed: {}", io::Error::last_os_error());
        }
    );
}

pub struct WindowsScreen {
    visible_choices: u16,
    start_line: u16,
    original_console_mode: DWORD,
    original_colors: WORD,
    input: Receiver<INPUT_RECORD>,
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
        let blank_line = repeat(' ').take((self.width() - 1) as usize).collect::<String>();
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
        let cursor_info = CONSOLE_CURSOR_INFO { dwSize: 100, bVisible: FALSE };
        win32!(SetConsoleCursorInfo(self.conout, &cursor_info));
    }

    fn write(&mut self, s: &str) {
        Self::write_to(self.conout, s);
    }

    fn write_red_inverted(&mut self, s: &str) {
        let orig = self.original_colors;
        const WHITE_ON_RED: WORD = BACKGROUND_RED as WORD;
        self.set_colors(WHITE_ON_RED);
        self.write(s);
        self.set_colors(orig);
    }

    fn write_red(&mut self, s: &str) {
        let orig = self.original_colors;
        const RED_ON_BLACK: WORD = FOREGROUND_RED as WORD;
        self.set_colors(RED_ON_BLACK);
        self.write(s);
        self.set_colors(orig);
    }

    fn write_inverted(&mut self, s: &str) {
        let orig = self.original_colors;
        const BLACK_ON_WHITE: WORD = (BACKGROUND_RED | BACKGROUND_GREEN | BACKGROUND_BLUE) as WORD;
        self.set_colors(BLACK_ON_WHITE);
        self.write(s);
        self.set_colors(orig);
    }

    fn get_buffered_keys(&mut self) -> Vec<Key> {
        let mut ret = Vec::new();
        while let Ok(event) = self.input.try_recv() {
            ret.push(WindowsScreen::translate_event(event, &mut self.shifted));
        }
        if ret.is_empty() {
            let event = self.input.recv().unwrap();
            ret.push(WindowsScreen::translate_event(event, &mut self.shifted));
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
        let size = ::std::mem::size_of::<FILE_NAME_INFO>();
        let mut name_info_bytes = vec![0u8; size + MAX_PATH];
        let stdin: HANDLE = unsafe { GetStdHandle(STD_INPUT_HANDLE) };
        if 0 == unsafe {
            GetFileInformationByHandleEx(
                stdin,
                FileNameInfo,
                &mut *name_info_bytes as *mut _ as *mut c_void,
                name_info_bytes.len() as u32)
        } {
            // On Windows (but not Cygwin), the above call fails if stdin is interactive.
            false
        } else {
            let name = unsafe {
                let name_info = *(name_info_bytes[0..size].as_ptr() as *const FILE_NAME_INFO);
                let name_bytes = &name_info_bytes[size..size + name_info.FileNameLength as usize];
                let name_u16 = from_raw_parts(name_bytes.as_ptr() as *const u16, name_bytes.len() / 2);
                OsString::from_wide(name_u16).as_os_str().to_string_lossy().into_owned()
            };
            name.contains("msys-") || name.contains("-pty") || name.contains("cygwin-")
        }
    }

    pub fn open_screen(desired_rows: u16) -> WindowsScreen {
        let mut orig_mode = Default::default();
        let conin: HANDLE;
        let conout: HANDLE;
        unsafe {
            const OPEN_EXISTING: DWORD = 3;
            let rw_access = GENERIC_READ | GENERIC_WRITE;
            conin = CreateFileA("CONIN$\0".as_ptr() as *const i8, rw_access, FILE_SHARE_READ, ptr::null_mut(), OPEN_EXISTING, 0, ptr::null_mut());
            conout = CreateFileA("CONOUT$\0".as_ptr() as *const i8, rw_access, FILE_SHARE_READ, ptr::null_mut(), OPEN_EXISTING, 0, ptr::null_mut());
        }

        if conin == INVALID_HANDLE_VALUE || conout == INVALID_HANDLE_VALUE {
            panic!("Unable to open console");
        }

        let (_, rows) = WindowsScreen::winsize(conout).unwrap();

        win32!(GetConsoleMode(conin, &mut orig_mode));
        let new_mode = orig_mode & !(ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT | ENABLE_PROCESSED_INPUT);
        win32!(SetConsoleMode(conin, new_mode));
        let mut default_cursor_info = CONSOLE_CURSOR_INFO { dwSize: 100, bVisible: TRUE };
        win32!(GetConsoleCursorInfo(conout, &mut default_cursor_info as PCONSOLE_CURSOR_INFO));

        let rx = WindowsScreen::spawn_input_thread(conin as usize);
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
            input: rx,
            conin,
            conout,
            default_cursor_info,
            shifted: false,
        }
    }

    // We have to take the conin handle as a usize instead of a *mut c_void in order to avoid a
    // lecture from the compiler about how the latter type cannot be safely sent between threads.
    // I'm not sure if a better solution exists at this time.
    fn spawn_input_thread(conin: usize) -> Receiver<INPUT_RECORD> {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            loop {
                let conin = conin as *mut c_void;
                let mut input_record = INPUT_RECORD::default();
                let mut events_read: DWORD = 0;
                win32!(ReadConsoleInputW(conin, &mut input_record as PINPUT_RECORD, 1, &mut events_read as LPDWORD));
                if events_read > 0 {
                    tx.send(input_record).unwrap();
                }
            }
        });

        rx
    }

    fn write_to(conout: HANDLE, s: &str) {
        let mut chars_written: DWORD = 0;
        let utf16 = s.encode_utf16().collect::<Vec<u16>>();
        let chars_to_write = utf16.len() as DWORD;
        win32!(WriteConsoleW(conout, utf16.as_ptr() as PVOID, chars_to_write, &mut chars_written as LPDWORD, ptr::null_mut()));
    }

    fn move_cursor(&mut self, line: u16, column: u16) {
        win32!(SetConsoleCursorPosition(self.conout, COORD { X: column as i16, Y: line as i16}));
    }

    fn set_colors(&mut self, colors: WORD) {
        win32!(SetConsoleTextAttribute(self.conout, colors));
    }

    fn translate_event(event: INPUT_RECORD, shifted: &mut bool) -> Key {
        use winapi::um::winuser::*;
        if event.EventType != KEY_EVENT {
            return Nothing;
        }

        let key_event = unsafe { event.Event.KeyEvent() };
        let vk_code = key_event.wVirtualKeyCode as i32;

        if vk_code == VK_SHIFT {
            *shifted = key_event.bKeyDown == TRUE;

            return Nothing;
        }

        if key_event.bKeyDown == FALSE {
            return Nothing;
        }

        if vk_code == VK_UP {
            Up
        } else if vk_code == VK_DOWN {
            Down
        } else if vk_code == VK_PRIOR {
            PgUp
        } else if vk_code == VK_NEXT {
            PgDown
        } else if vk_code == VK_HOME {
            Home
        } else if vk_code == VK_END {
            End
        } else if vk_code == VK_TAB {
            if *shifted { ShiftTab } else { Tab }
        } else if vk_code == VK_BACK {
            Backspace
        } else if vk_code == VK_RETURN {
            Enter
        } else if vk_code == VK_ESCAPE {
            Control('g')
        } else {
            let byte = unsafe { *key_event.uChar.UnicodeChar() };
            if byte & 96 == 0 {
                Control(((byte + 96u16) as u8) as char)
            } else {
                Char((byte as u8) as char)
            }
        }
    }

    fn get_cursor_pos(handle: HANDLE) -> (u16, u16) {
        let mut buffer_info = Default::default();
        win32!(GetConsoleScreenBufferInfo(handle, &mut buffer_info));
        let cursor_pos = buffer_info.dwCursorPosition;
        (cursor_pos.X as u16, cursor_pos.Y as u16)
    }

    fn get_original_colors(handle: HANDLE) -> WORD {
        let mut buffer_info = Default::default();
        win32!(GetConsoleScreenBufferInfo(handle, &mut buffer_info));
        buffer_info.wAttributes
    }

    fn winsize(conout: HANDLE) -> Option<(u16, u16)> {
        let mut buffer_info = Default::default();
        let result = unsafe { GetConsoleScreenBufferInfo(conout, &mut buffer_info) };
        if result != 0 {
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
        let mut buffer_info = Default::default();
        win32!(GetConsoleScreenBufferInfo(conout, &mut buffer_info));
        buffer_info.srWindow.Top as u16
    }
}

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

#[cfg(test)]
mod tests {
    use winapi::um::processenv::GetStdHandle;
    use winapi::um::winbase::STD_OUTPUT_HANDLE;
    use super::{WindowsScreen, get_start_line};

    #[test]
    fn winsize_test() {
        // AppVeyor builds run without a console, making this test impossible.
        if option_env!("APPVEYOR").is_some() || option_env!("TRAVIS").is_some() {
            // TODO: It should be made obvious from the output that this test was skipped
            return;
        }
        let conout = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) };
        let (cols, rows) = WindowsScreen::winsize(conout).expect("Failed to get window size!");
        // We don't know the window size a priori, but we can at least
        // assert that it is within some kind of sensible range.
        assert!(cols > 40);
        assert!(rows > 40);
        assert!(cols < 1000);
        assert!(rows < 1000);
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
