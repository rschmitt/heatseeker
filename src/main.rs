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
  let initial_search = args.initial_search.clone();
  if args.use_first {
    let matches = matching::compute_matches(&choices, initial_search.as_slice());
    println!("{}", matches[0]);
    return;
  } else {
    event_loop(choices, initial_search.as_slice());
  }
}

fn event_loop(choices: Vec<String>, initial_search: &str) {
  let mut screen = Screen::open_screen();
  let mut index = 0;

  let mut search = Search {
    query: initial_search.to_string(),
    matches: Vec::new(),
    stale: true,
  };
  loop {
    if search.stale {
      search.matches = matching::compute_matches(&choices, search.query.as_slice());
      search.stale = false;
    }

    draw_screen(&mut screen, &search.matches, search.query.as_slice(), choices.len(), index);

    let chars = screen.get_buffered_keys();
    for char in chars.iter() {
      match *char {
        Char(x) => {
          search.query.push(x);
          index = 0;
          search.stale = true;
        }
        Backspace => { search.backspace(); }
        Control('h') => { search.backspace(); }
        Control('u') => { search.clear_query(); }
        Control('c') => { return; }
        Control('n') => { index = min(index + 1, min(screen.visible_choices as usize - 1, search.matches.len() - 1)); }
        Control('p') => { index = if index == 0 { 0 } else { index - 1 }; }
        Enter => {
          screen.move_cursor_to_bottom();
          if search.stale {
            search.matches = matching::compute_matches(&choices, search.query.as_slice());
          }
          println!("{}", search.matches[index]);
          return;
        }
        _ => panic!("Unexpected input"),
      }
    }
  }
}

struct Search<'a> {
  query: String,
  matches: Vec<&'a String>,
  stale: bool,
}

impl<'a> Search<'a> {
  fn backspace(&mut self) {
    self.query.pop();
    self.stale = true;
  }

  fn clear_query(&mut self) {
    self.query.clear();
    self.stale = true;
  }
}

fn draw_screen(screen: &mut Screen, matches: &Vec<&String>, search: &str, choices: usize, index: usize) {
  screen.hide_cursor();
  screen.blank_screen();
  screen.write(format!("> {} ({} choices)\n", search, choices).as_slice());

  print_matches(screen, matches, index);

  let start = screen.start_line;
  screen.move_cursor(start, 2 + search.len() as u16);
  screen.show_cursor();
}

fn print_matches(screen: &mut Screen, matches: &Vec<&String>, index: usize) {
  let mut i = 1;
  for choice in matches.iter() {
    if i == index + 1 {
      screen.write_inverted(choice.as_slice());
    } else {
      screen.write(choice.as_slice());
    }
    if i >= screen.visible_choices as usize {
      return;
    } else {
      screen.write("\n");
    }
    i += 1;
  }
}

fn read_choices() -> Vec<String> {
  let mut stdin = io::stdio::stdin();
  let mut lines = Vec::new();

  while let Ok(mut s) = stdin.read_line() {
    trim(&mut s);
    lines.push(s);
  }

  lines
}

fn trim(s: &mut String) {
  while let Some(x) = s.pop() {
    if x != '\n' {
      s.push(x);
      return;
    }
  }
}
