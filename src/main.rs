#![allow(unstable, dead_code)]

extern crate getopts;
extern crate libc;

mod ansi;
mod matching;
mod screen;

use std::os;
use std::io;
use std::cmp::min;
use screen::Screen;
use screen::Key::*;

fn main() {
  let input_args = os::args();

  let args = match parse_args(&input_args) {
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

    match screen.tty.getchar() {
      Char(x) => search.push(x),
      Backspace => { search.pop(); }
      Control('h') => { search.pop(); }
      Control('u') => { search.clear(); }
      Control('c') => { return; }
      Control('n') => { index += 1; }
      Control('p') => { index -= 1; }
      Enter => {
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

struct Args {
  initial_search: String,
  help: bool,
}

fn parse_args(args: &Vec<String>) -> Option<Args> {
  let opts = [
    getopts::optflag("h", "help", "Show this message"),
    getopts::optopt("s", "search", "Specify an initial search string", "SEARCH"),
  ];

  let matches = match getopts::getopts(args.tail(), &opts) {
    Ok(m) => m,
    Err(f) => {
      println!("{}", f);
      print_usage(args[0].as_slice(), &opts);
      return None;
    }
  };

  let initial_search = match matches.opt_str("search") {
    Some(x) => x.clone(),
    None => String::from_str(""),
  };

  let help = matches.opt_present("help");
  if help {
    print_usage(args[0].as_slice(), &opts);
  }

  Some(Args { initial_search: initial_search, help: help })
}

fn print_usage(program: &str, opts: &[getopts::OptGroup]) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", getopts::usage(brief.as_slice(), opts));
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
