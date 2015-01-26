#![allow(unstable, dead_code)]

extern crate libc;

mod args;
mod ansi;
mod matching;
mod screen;

use std::os;
use std::io;
use std::cmp::min;
use screen::Screen;
use screen::Key::*;

fn main() {
  let args = match args::parse_args() {
    Some(args) => args,
    None => {
      os::set_exit_status(1);
      return;
    },
  };

  if args.help { return; }

  let mut screen = Screen::open_screen();
  let mut index = 0;

  let choices = read_choices();
  let visible_choices = min(20, screen.height - 1);

  let mut search: String = args.initial_search.clone();
  let start_line = screen.height - visible_choices - 1;
  loop {
    blank_screen(&mut screen, start_line);
    screen.move_cursor(start_line, 0);
    let matches = matching::compute_matches(&choices, search.as_slice());
    let mut i = 1;
    screen.tty.writeln(format!("> {} ({} choices)", search.as_slice(), choices.len()).as_slice());
    for choice in matches.iter() {
      if i == index + 1 {
        screen.tty.write(ansi::inverse().as_slice());
        screen.tty.writeln(choice.as_slice());
        screen.tty.write(ansi::reset().as_slice());
      } else {
        screen.tty.writeln(choice.as_slice());
      }
      if i >= visible_choices as usize {
        break;
      }
      i += 1;
    }

    screen.move_cursor(start_line, 2 + search.len() as u16);

    match screen.tty.getchar() {
      Char(x) => search.push(x),
      Backspace => { search.pop(); }
      Control('h') => { search.pop(); }
      Control('u') => { search.clear(); }
      Control('c') => { return; }
      Control('n') => { index += 1; }
      Control('p') => { index -= 1; }
      Enter => {
        screen.move_cursor(start_line + visible_choices, 0);
        println!("{}", matches[index]);
        break;
      }
      _ => panic!("Unexpected input"),
    }
  }
}

fn blank_screen(screen: &mut screen::Screen, start_line: u16) {
  screen.move_cursor(start_line, 0);
  let mut i = 0;
  while i < screen.height {
    let mut j = 0;
    while j < screen.width {
      screen.tty.write(" ".as_bytes());
      j += 1;
    }
    i += 1;
  }
}

fn read_choices() -> Vec<String> {
  let mut stdin = io::stdio::stdin();
  let mut lines = Vec::new();

  loop {
    match stdin.read_line() {
      Ok(x) => lines.push(String::from_str(x.as_slice().trim())),
      Err(_) => break,
    }
  }

  lines
}
