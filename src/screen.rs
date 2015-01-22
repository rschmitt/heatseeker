use std::io::{File, Open, Read, Write, IoResult};
use libc::{c_ushort, c_int, c_ulong, STDOUT_FILENO};
use std::os::unix::AsRawFd;

pub struct Terminal {
  input: File,
  output: File,
}

impl Terminal {
  pub fn open_terminal() -> Terminal {
    let term_path = Path::new("/dev/tty");
    let input_file = File::open_mode(&term_path, Open, Read).unwrap();
    let output_file = File::open_mode(&term_path, Open, Write).unwrap();
    Terminal { input: input_file, output: output_file }
  }

  pub fn writeln(&mut self, s: &str) {
    self.output.write_line(s);
  }

  pub fn winsize(&self) -> Option<(u16, u16)> {
    extern { fn ioctl(fd: c_int, request: c_ulong, ...) -> c_int; }
    const TIOCGWINSZ: c_ulong = 0x40087468;

    #[repr(C)]
    struct TermSize {
      rows: c_ushort,
      cols: c_ushort,
      x: c_ushort,
      y: c_ushort,
    }

    let size = TermSize { rows: 0, cols: 0, x: 0, y: 0 };
    if unsafe { ioctl(self.output.as_raw_fd(), TIOCGWINSZ, &size) } == 0 {
      Some((size.cols as u16, size.rows as u16))
    } else {
      None
    }
  }
}
