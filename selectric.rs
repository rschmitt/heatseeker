extern crate getopts;

use std::ascii::AsciiExt;
use std::cmp::min as min;
use std::os;
use std::io;

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

  println!("");
  println!("Choices:");
  for choice in choices.iter() {
    println!("{}", choice);
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
      print_usage(args[0].as_slice(), opts);
      return None;
    }
  };

  let initial_search = match matches.opt_str("search") {
    Some(x) => x.clone(),
    None => String::from_str(""),
  };

  let help = matches.opt_present("help");
  if help {
    print_usage(args[0].as_slice(), opts);
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

fn score(choice: &str, query: &str) -> f64 {
  if query.len() == 0 {
    return 1.0;
  }
  if choice.len() == 0 {
    return 0.0;
  }

  let query = query.to_ascii_lower();
  let choice = choice.to_ascii_lower();

  match compute_match_length(choice.as_slice(), query.as_slice()) {
    None => return 0.0,
    Some(match_length) => {
      let score = query.len() as f64 / match_length as f64;
      return score as f64 / choice.len() as f64;
    }
  }
}

fn compute_match_length(string: &str, chars: &str) -> Option<uint> {
  let first_char = chars.char_at(0);
  let rest = chars.slice(1, chars.len());
  let indices = find_char_in_string(string, first_char);

  let mut current_min: Option<uint> = None;
  for i in indices.iter() {
    let last_index = find_end_of_match(string, rest, *i);
    if last_index.is_some() {
      let idx = last_index.unwrap() - *i + 1;
      if current_min.is_some() {
        let cm = min(current_min.unwrap(), idx);
        current_min = Some(cm);
      } else {
        current_min = Some(idx)
      }
    }
  }
  return current_min;
}

fn find_char_in_string(string: &str, char: char) -> Vec<uint> {
  let mut indices = Vec::new();
  let mut i = 0;
  for c in string.chars() {
    if c == char {
      indices.push(i);
    }
    i = i + 1;
  }
  return indices;
}

fn find_end_of_match(string: &str, rest_of_query: &str, first_index: uint) -> Option<uint> {
  let mut last_index = first_index + 1;
  for c in rest_of_query.chars() {
    let current_substring = string.slice_chars(last_index, string.len());
    match current_substring.find(c) {
      None => return None,
      Some(ref i) => {
        last_index += *i + 1;
      }
    }
  }
  return Some(last_index - 1);
}

#[test]
fn find_end_of_match_test() {
  assert_eq!(find_end_of_match("a", "a", 0), None);
  assert_eq!(find_end_of_match("ba", "a", 1), None);
  assert_eq!(find_end_of_match("aaa", "aa", 0), Some(2));
  assert_eq!(find_end_of_match("aaa", "b", 0), None);
  assert_eq!(find_end_of_match("this is a long match", "this is a match", 0), None);
  assert_eq!(find_end_of_match("this is a long match", "his is a match", 0), Some(19));
}

#[test]
fn basic_scoring() {
  assert_eq!(score("", "a"), 0.0);
  assert_eq!(score("a", ""), 1.0);
  assert_eq!(score("short", "longer"), 0.0);
  assert_eq!(score("a", "b"), 0.0);
  assert_eq!(score("ab", "ac"), 0.0);

  assert!(score("a", "a") > 0.0);
  assert!(score("ab", "a") > 0.0);
  assert!(score("ba", "a") > 0.0);
  assert!(score("bab", "a") > 0.0);
  assert!(score("babababab", "aaaa") > 0.0);

  assert_eq!(score("a", "a"), 1_f64 / "a".len() as f64);
  assert_eq!(score("ab", "ab"), 0.5);
  assert_eq!(score("a long string", "a long string"), 1_f64 / "a long string".len() as f64);
  assert_eq!(score("spec/search_spec.rb", "sear"), 1_f64 / "spec/search_spec.rb".len() as f64);
}

#[test]
fn character_matching() {
  assert!(score("/! symbols $^", "/!$^") > 0.0);

  assert_eq!(score("a", "A"), 1.0);
  assert_eq!(score("A", "a"), 1.0);

  assert_eq!(score("a", "aa"), 0.0);
}

#[test]
fn match_equality() {
  assert!(score("selecta.gemspec", "asp") > score("algorithm4_spec.rb", "asp"));
  assert!(score("README.md", "em") > score("benchmark.rb", "em"));
  assert!(score("search.rb", "sear") > score("spec/search_spec.rb", "sear"));

  assert!(score("fbb", "fbb") > score("foo bar baz", "fbb"));
  assert!(score("foo", "foo") > score("longer foo", "foo"));
  assert!(score("foo", "foo") > score("foo longer", "foo"));
  assert!(score("1/2/3/4", "1/2/3") > score("1/9/2/3/4", "1/2/3"));

  assert!(score("long 12 long", "12") > score("1 long 2", "12"));

  assert_eq!(score("121padding2", "12"), 1.0 / "121padding2".len() as f64);
  assert_eq!(score("1padding212", "12"), 1.0 / "1padding212".len() as f64);
}
