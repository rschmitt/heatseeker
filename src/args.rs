extern crate getopts;

use std::env;

pub struct Args {
    pub initial_search: String,
    pub help: bool,
    pub use_first: bool,
    pub version: bool,
}

pub fn parse_args() -> Option<Args> {
    let mut os_args = env::args();
    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "Show this message");
    opts.optflag("v", "version", "Show version");
    opts.optflagopt("s", "search", "Specify an initial search string", "SEARCH");
    opts.optflag("f", "first", "Automatically select the first match");

    let mut args = Vec::new();
    while let Some(os_arg) = os_args.next() {
        args.push(os_arg);
    }

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            println!("{}", f);
            print_usage(&args[0], &opts);
            return None;
        }
    };

    let initial_search = match matches.opt_str("search") {
        Some(x) => x.clone(),
        None => "".to_string(),
    };

    let help = matches.opt_present("help");
    if help {
        print_usage(&args[0], &opts);
    }

    let version = matches.opt_present("version");

    Some(Args {
        initial_search: initial_search,
        help: help,
        use_first: matches.opt_present("first"),
        version: version,
    })
}

fn print_usage(program: &str, opts: &getopts::Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

