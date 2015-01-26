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

  let choices = read_choices();

  let mut screen = Screen::open_screen();
  let mut index = 0;

  let visible_choices = min(20, screen.height - 1);

  let mut search = args.initial_search.clone();
  let start_line = screen.height - visible_choices - 1;
  loop {
    screen.hide_cursor();
    screen.blank_screen(start_line);
    screen.move_cursor(start_line, 0);
    let matches = matching::compute_matches(&choices, search.as_slice());
    let mut i = 1;
    screen.write(format!("> {} ({} choices)\n", search.as_slice(), choices.len()).as_slice());
    for choice in matches.iter() {
      if i == index + 1 {
        screen.write_inverted(choice.as_slice());
      } else {
        screen.write(choice.as_slice());
      }
      if i >= visible_choices as usize {
        break;
      } else {
        screen.write("\n");
      }
      i += 1;
    }

    screen.move_cursor(start_line, 2 + search.len() as u16);
    screen.show_cursor();

    match screen.tty.getchar() {
      Char(x) => {
        search.push(x);
        index = 0;
      }
      Backspace => { search.pop(); }
      Control('h') => { search.pop(); }
      Control('u') => { search.clear(); }
      Control('c') => { return; }
      Control('n') => { index = min(index + 1, min(visible_choices as usize - 1, matches.len() - 1)); }
      Control('p') => { index = if index == 0 { 0 } else { index - 1 }; }
      Enter => {
        screen.move_cursor(start_line + visible_choices, 0);
        screen.write("\n");
        println!("{}", matches[index]);
        break;
      }
      _ => panic!("Unexpected input"),
    }
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
