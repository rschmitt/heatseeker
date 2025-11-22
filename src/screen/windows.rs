use super::Key;
use super::Key::*;
use super::Screen;
use crate::NEWLINE;
use crate::ansi;
use std::cmp::min;
use std::str;

use windows::Win32::Foundation::{HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_GENERIC_READ, FILE_GENERIC_WRITE, FILE_SHARE_READ,
    FILE_SHARE_WRITE, OPEN_EXISTING,
};
use windows::Win32::System::Console::{
    CONSOLE_MODE, CONSOLE_SCREEN_BUFFER_INFO, ENABLE_VIRTUAL_TERMINAL_PROCESSING, GetConsoleMode,
    GetConsoleScreenBufferInfo, INPUT_RECORD, KEY_EVENT, ReadConsoleInputW, SetConsoleMode,
    WINDOW_BUFFER_SIZE_EVENT, WriteConsoleW,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    VK_BACK, VK_DOWN, VK_END, VK_ESCAPE, VK_HOME, VK_NEXT, VK_PRIOR, VK_RETURN, VK_SHIFT, VK_TAB,
    VK_UP,
};
use windows::core::w;

macro_rules! win32 {
    ($expr:expr) => {{
        let result = unsafe { $expr };
        if result.is_err() {
            panic!("win32 call failed: {}", std::io::Error::last_os_error());
        }
    }};
}

pub struct WindowsScreen {
    tty: Terminal,
    desired_rows: u16,
    pending_resize: bool,
}

impl Screen for WindowsScreen {
    fn desired_rows(&self) -> u16 {
        self.desired_rows
    }

    fn winsize(&self) -> Option<(u16, u16)> {
        self.tty.winsize()
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        self.tty.write(bytes);
    }

    fn flush(&mut self) {
        self.tty.flush();
    }

    fn get_buffered_keys(&mut self) -> Vec<Key> {
        if self.pending_resize {
            self.pending_resize = false;
            self.blank_entire_screen();
            return vec![Nothing];
        }

        match self.tty.read_event() {
            TerminalMessage::Keys(keys, resize) => {
                if resize {
                    self.pending_resize = true;
                }
                keys
            }
            TerminalMessage::Resize => {
                self.blank_entire_screen();
                vec![Nothing]
            }
        }
    }
}

impl WindowsScreen {
    pub fn open_screen(desired_rows: u16) -> WindowsScreen {
        let mut tty = Terminal::open_terminal();
        tty.write(ansi::reset());
        let (_, rows) = tty.winsize().unwrap();
        let visible_choices = min(desired_rows, rows.saturating_sub(1));
        for _ in 0..visible_choices {
            tty.write(NEWLINE.as_bytes());
        }
        tty.write(ansi::save_cursor());

        WindowsScreen {
            tty,
            desired_rows,
            pending_resize: false,
        }
    }
}

struct Terminal {
    conin: HANDLE,
    conout: HANDLE,
    output_buf: Vec<u8>,
    original_input_mode: CONSOLE_MODE,
    original_output_mode: CONSOLE_MODE,
    shifted: bool,
}

enum TerminalMessage {
    Keys(Vec<Key>, bool),
    Resize,
}

impl Terminal {
    fn open_terminal() -> Terminal {
        let rw_access = FILE_GENERIC_READ | FILE_GENERIC_WRITE;
        let conin;
        let conout;
        unsafe {
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
        assert!(
            conin != INVALID_HANDLE_VALUE && conout != INVALID_HANDLE_VALUE,
            "Unable to open console"
        );

        let mut input_mode = CONSOLE_MODE::default();
        let mut output_mode = CONSOLE_MODE::default();
        win32!(GetConsoleMode(conin, &raw mut input_mode));
        win32!(GetConsoleMode(conout, &raw mut output_mode));

        let mut vt_output_mode = output_mode;
        vt_output_mode |= ENABLE_VIRTUAL_TERMINAL_PROCESSING;
        win32!(SetConsoleMode(conout, vt_output_mode));

        Terminal {
            conin,
            conout,
            output_buf: Vec::new(),
            original_input_mode: input_mode,
            original_output_mode: output_mode,
            shifted: false,
        }
    }

    fn write(&mut self, s: &[u8]) {
        self.output_buf.extend_from_slice(s);
    }

    fn flush(&mut self) {
        if self.output_buf.is_empty() {
            return;
        }
        let text = str::from_utf8(&self.output_buf).unwrap();
        let utf16: Vec<u16> = text.encode_utf16().collect();
        let mut chars_written = 0;
        win32!(WriteConsoleW(
            self.conout,
            &utf16,
            Some(&raw mut chars_written),
            None
        ));
        self.output_buf.clear();
    }

    fn winsize(&self) -> Option<(u16, u16)> {
        console_winsize(self.conout)
    }

    fn read_event(&mut self) -> TerminalMessage {
        let mut buffer = [INPUT_RECORD::default(); 32];
        loop {
            let mut events_read = 0;
            win32!(ReadConsoleInputW(
                self.conin,
                &mut buffer,
                &raw mut events_read
            ));
            let mut keys = Vec::new();
            let mut resize = false;
            for record in buffer.iter().take(events_read as usize) {
                match u32::from(record.EventType) {
                    KEY_EVENT => {
                        if let Some(k) = self.translate_event(record) {
                            keys.push(k);
                        }
                    }
                    WINDOW_BUFFER_SIZE_EVENT => resize = true,
                    _ => {}
                }
            }
            if !keys.is_empty() {
                return TerminalMessage::Keys(keys, resize);
            }
            if resize {
                return TerminalMessage::Resize;
            }
        }
    }

    fn translate_event(&mut self, event: &INPUT_RECORD) -> Option<Key> {
        let key_event = unsafe { event.Event.KeyEvent };
        let vk_code = key_event.wVirtualKeyCode;

        if vk_code == VK_SHIFT.0 {
            self.shifted = key_event.bKeyDown.as_bool();
            return None;
        }

        if !key_event.bKeyDown.as_bool() {
            return None;
        }

        let key = if vk_code == VK_UP.0 {
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
            if self.shifted { ShiftTab } else { Tab }
        } else if vk_code == VK_BACK.0 {
            Backspace
        } else if vk_code == VK_RETURN.0 {
            Enter
        } else if vk_code == VK_ESCAPE.0 {
            Control('g')
        } else {
            let ch = u32::from(unsafe { key_event.uChar.UnicodeChar });
            if ch == 0 {
                return None;
            }
            if ch & 96 == 0 {
                let c = ((ch as u16 + 96) as u8) as char;
                Control(c)
            } else {
                match char::from_u32(ch) {
                    Some(c) => Char(c),
                    None => return None,
                }
            }
        };
        Some(key)
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        self.flush();
        let _ = unsafe { SetConsoleMode(self.conin, self.original_input_mode) };
        let _ = unsafe { SetConsoleMode(self.conout, self.original_output_mode) };
    }
}

fn console_winsize(conout: HANDLE) -> Option<(u16, u16)> {
    let mut buffer_info = CONSOLE_SCREEN_BUFFER_INFO::default();
    let result = unsafe { GetConsoleScreenBufferInfo(conout, &raw mut buffer_info) };
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

#[cfg(test)]
mod tests {
    use super::console_winsize;
    use windows::Win32::System::Console::{GetStdHandle, STD_OUTPUT_HANDLE};

    #[test]
    fn winsize_test() {
        if option_env!("APPVEYOR").is_some() || option_env!("TRAVIS").is_some() {
            return;
        }
        let conout = unsafe { GetStdHandle(STD_OUTPUT_HANDLE).unwrap() };
        let (cols, rows) = console_winsize(conout).expect("failed to get window size");
        assert!(cols > 40 && cols < 1000);
        assert!(rows > 10 && rows < 1000);
    }
}
