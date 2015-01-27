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

  let mut search = Search {
    choices: &choices,
    query: initial_search.to_string(),
    matches: Vec::new(),
    stale: true,
    index: 0,
  };
  loop {
    search.recompute_matches();
    draw_screen(&mut screen, &search);

    let chars = screen.get_buffered_keys();
    for char in chars.iter() {
      match *char {
        Char(x) => search.append(x),
        Backspace => search.backspace(),
        Control('h') => search.backspace(),
        Control('u') => search.clear_query(),
        Control('c') => return,
        Control('n') => search.down(screen.visible_choices),
        Control('p') => search.up(),
        Enter => {
          screen.move_cursor_to_bottom();
          println!("{}", search.get_selection());
          return;
        }
        _ => panic!("Unexpected input"),
      }
    }
  }
}

struct Search<'a> {
  choices: &'a Vec<String>,
  query: String,
  matches: Vec<&'a String>,
  stale: bool,
  index: usize,
}

impl<'a> Search<'a> {
  fn up(&mut self) {
    self.index = if self.index == 0 { 0 } else { self.index - 1 };
  }

  fn down(&mut self, visible_choices: u16) {
    let limit = min(visible_choices as usize - 1, self.matches.len() - 1);
    self.index = min(self.index + 1, limit);
  }

  fn backspace(&mut self) {
    self.query.pop();
    self.stale = true;
    self.index = 0;
  }

  fn append(&mut self, c: char) {
    self.query.push(c);
    self.stale = true;
  }

  fn clear_query(&mut self) {
    self.query.clear();
    self.stale = true;
  }

  fn recompute_matches(&mut self) {
    if self.stale {
      self.matches = matching::compute_matches(self.choices, self.query.as_slice());
      self.stale = false;
    }
  }

  fn get_selection(&mut self) -> &String {
    self.recompute_matches();
    self.matches[self.index]
  }
}

fn draw_screen(screen: &mut Screen, search: &Search) {
  screen.hide_cursor();
  screen.blank_screen();
  screen.write(format!("> {} ({} choices)\n", search.query, search.matches.len()).as_slice());

  print_matches(screen, &search.matches, search.index);

  let start = screen.start_line;
  screen.move_cursor(start, 2 + search.query.len() as u16);
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
