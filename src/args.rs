extern crate getopts;

use std::os;

struct Args {
    pub initial_search: String,
    pub help: bool,
    pub use_first: bool,
}

pub fn parse_args() -> Option<Args> {
    let args = os::args();
    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "Show this message");
    opts.optflagopt("s", "search", "Specify an initial search string", "SEARCH");
    opts.optflag("f", "first", "Automatically select the first match");

    let matches = match opts.parse(args.tail()) {
    // let matches = match opts.getopts(args.tail(), &opts) {
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

    Some(Args {
        initial_search: initial_search,
        help: help,
        use_first: matches.opt_present("first"),
    })
}

fn print_usage(program: &str, opts: &getopts::Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(brief.as_slice()));
}

