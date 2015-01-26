extern crate getopts;

use std::os;

struct Args {
  pub initial_search: String,
  pub help: bool,
}

pub fn parse_args() -> Option<Args> {
  let args = os::args();
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

