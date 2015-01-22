extern crate getopts;
extern crate libc;

use std::os;
use std::io;

mod matching;
mod screen;

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

  println!("Initial search: \"{}\"", args.initial_search);

  let choices = read_choices();

  let mut terminal = screen::Terminal::open_terminal();

  println!("");
  println!("Choices:");
  for choice in choices.iter() {
    println!("{}", choice);
  }
  println!("");
  println!("Filtered choices:");
  for choice in matching::compute_matches(&choices, args.initial_search.as_slice()).iter() {
    println!("{}", choice);
  }
  let (col, row) = terminal.winsize().unwrap();
  terminal.writeln(format!("{}x{} terminal", col, row).as_slice());
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
