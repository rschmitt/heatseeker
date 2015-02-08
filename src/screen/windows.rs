#![cfg(windows)]

extern crate libc;
extern crate winapi;
extern crate "kernel32-sys" as kernel32;

use self::kernel32::*;
use self::winapi::*;
use std::ptr;
use std::iter::repeat;
use std::cmp::min;
use screen::Key;
use screen::Key::*;

macro_rules! win32 {
    ($funcall:expr) => (
        if unsafe { $funcall } == 0 {
            panic!("Win32 call failed");
        }
    );
}

pub struct Screen {
    pub height: u16,
    pub width: u16,
    pub visible_choices: u16,
    pub start_line: u16,
    original_console_mode: DWORD,
    original_colors: WORD,
    conin: HANDLE,
    conout: HANDLE,
}

impl Drop for Screen {
    fn drop(&mut self) {
        win32!(SetConsoleMode(self.conout, self.original_console_mode));
    }
}

impl Screen {
    pub fn open_screen() -> Screen {
        let mut orig_mode;
        let conin: HANDLE;
        let conout: HANDLE;
        unsafe {
            // Unlike in Linux, we *could* use the default stdin/stdout file handles to talk
            // directly to the console. However, we need the default stdin handle for the list of
            // choices, and we need to write just the final selection to stdout. Therefore we have
            // to explicitly create new handles to talk to the console. The gory details are
            // available at:
            //
            // https://msdn.microsoft.com/en-us/library/windows/desktop/ms682075%28v=vs.85%29.aspx
            // http://stackoverflow.com/questions/377152/what-does-createfileconin-do
            const OPEN_EXISTING: DWORD = 3;
            let rw_access = GENERIC_READ | GENERIC_WRITE;
            conin = CreateFileA("CONIN$\0".as_ptr() as *const i8, rw_access, FILE_SHARE_READ, ptr::null_mut(), OPEN_EXISTING, 0, ptr::null_mut());
            conout = CreateFileA("CONOUT$\0".as_ptr() as *const i8, rw_access, FILE_SHARE_READ, ptr::null_mut(), OPEN_EXISTING, 0, ptr::null_mut());
            orig_mode = ::std::mem::uninitialized();
        }

        if conin == INVALID_HANDLE_VALUE || conout == INVALID_HANDLE_VALUE {
            panic!("Unable to open console");
        }

        let (cols, rows) = Screen::winsize(conout).unwrap();

        win32!(GetConsoleMode(conout, &mut orig_mode));
        let new_mode = orig_mode & !(ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT);
        win32!(SetConsoleMode(conin, new_mode));

        let visible_choices = min(20, rows - 1);
        let start_line = rows - visible_choices - 1;
        let original_colors = Screen::get_original_colors(conout);
        Screen {
            height: rows,
            width: cols,
            visible_choices: visible_choices,
            start_line: start_line,
            original_console_mode: orig_mode,
            original_colors: original_colors,
            conin: conin,
            conout: conout,
        }
    }

    pub fn move_cursor(&mut self, line: u16, column: u16) {
        win32!(SetConsoleCursorPosition(self.conout, COORD { X: column as i16, Y: line as i16}));
    }

    pub fn move_cursor_to_bottom(&mut self) {
        let end_line = self.start_line + self.visible_choices;
        self.move_cursor(end_line, 0);
        self.write("\n");
    }

    pub fn blank_screen(&mut self) {
        let start_line = self.start_line;
        self.move_cursor(start_line, 0);
        let blank_line = repeat(' ').take(self.width as usize).collect::<String>();
        for _ in 0..self.height {
            self.write(&blank_line);
        }
        self.move_cursor(start_line, 0);
    }

    pub fn show_cursor(&mut self) {
        let cursor_info = CONSOLE_CURSOR_INFO { dwSize: 100, bVisible: TRUE };
        win32!(SetConsoleCursorInfo(self.conout, &cursor_info));
    }

    pub fn hide_cursor(&mut self) {
        let cursor_info = CONSOLE_CURSOR_INFO { dwSize: 100, bVisible: FALSE };
        win32!(SetConsoleCursorInfo(self.conout, &cursor_info));
    }

    pub fn write(&mut self, s: &str) {
        let mut copy = s.to_string();
        let len: DWORD = copy.len() as DWORD;
        copy.push('\0');
        let mut bytes_written: DWORD = 0;
        win32!(WriteFile(self.conout, copy.as_ptr() as PVOID, len, &mut bytes_written as LPDWORD, ptr::null_mut()));
    }

    pub fn write_inverted(&mut self, s: &str) {
        let orig = self.original_colors;
        const BLACK_ON_WHITE: WORD = (BACKGROUND_RED | BACKGROUND_GREEN | BACKGROUND_BLUE) as WORD;
        self.set_colors(BLACK_ON_WHITE);
        self.write(s);
        self.set_colors(orig);
    }

    fn set_colors(&mut self, colors: WORD) {
        win32!(SetConsoleTextAttribute(self.conout, colors));
    }

    // Currently the Windows implementation does not buffer, so this function just performs a
    // blocking read of a single key.
    pub fn get_buffered_keys(&mut self) -> Vec<Key> {
        let mut buf: Vec<u16> = repeat(0u16).take(0x1000).collect();
        let mut chars_read: DWORD = 0;
        win32!(ReadFile(self.conin, buf.as_mut_ptr() as LPVOID, 1, &mut chars_read as LPDWORD, ptr::null_mut()));

        let mut ret = Vec::new();
        for i in 0..chars_read {
            ret.push(Screen::translate_byte(buf[i as usize]));
        }
        ret
    }

    fn translate_byte(byte: u16) -> Key {
        if byte == '\r' as u16 {
            Enter
        } else if byte == 127 {
            Backspace
        } else if byte & 96 == 0 {
            Control(((byte + 96u16) as u8) as char)
        } else {
            Char((byte as u8) as char)
        }
    }

    fn get_original_colors(handle: HANDLE) -> WORD {
        let mut buffer_info = unsafe { ::std::mem::uninitialized() };
        win32!(GetConsoleScreenBufferInfo(handle, &mut buffer_info));
        buffer_info.wAttributes
    }

    fn winsize(conin: HANDLE) -> Option<(u16, u16)> {
        let mut buffer_info = unsafe { ::std::mem::uninitialized() };
        let result = unsafe { GetConsoleScreenBufferInfo(conin, &mut buffer_info) };
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
}

#[test]
fn winsize_test() {
    let conout = unsafe { kernel32::GetStdHandle(-11) };
    let (cols, rows) = Screen::winsize(conout).expect("Failed to get window size!");
    // We don't know the window size a priori, but we can at least
    // assert that it is within some kind of sensible range.
    assert!(cols > 40);
    assert!(rows > 40);
    assert!(cols < 1000);
    assert!(rows < 1000);
}
