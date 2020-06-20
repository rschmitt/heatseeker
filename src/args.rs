extern crate getopts;

use std::env;

pub struct Args {
    pub initial_search: String,
    pub help: bool,
    pub use_first: bool,
    pub version: bool,
    pub full_screen: bool,
    pub filter_only: bool,
}

pub fn parse_args() -> Option<Args> {
    let os_args = env::args();
    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "Show this message");
    opts.optflag("v", "version", "Show version");
    opts.optflagopt("s", "search", "Specify an initial search string", "SEARCH");
    opts.optflag("f", "first", "Automatically select the first match");
    opts.optflag("F", "full-screen", "Use the entire screen in order to display as many choices as possible");
    opts.optflag("", "filter-only", "Just filter choices without ranking them");

    let args = os_args.collect::<Vec<_>>();

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
    let full_screen = matches.opt_present("full-screen");

    Some(Args {
        initial_search,
        help,
        use_first: matches.opt_present("first"),
        version,
        full_screen,
        filter_only: matches.opt_present("filter-only"),
    })
}

fn print_usage(program: &str, opts: &getopts::Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

