extern crate time;

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use time::{strftime, now};
use std::process::Command;

fn main() {
    let basedir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let version = env!("CARGO_PKG_VERSION");
    let dest_path = Path::new(&basedir).join("src/version.rs");
    let mut f = File::create(&dest_path).unwrap();

    let timestamp = strftime("%F %H:%M:%S %z", &now()).unwrap();
    let commit = get_head_commit();

    let contents = format!(
"pub const VERSION: &'static str = \"{}\";
pub const TIMESTAMP: &'static str = \"{}\";
pub const COMMIT: &'static str = \"{}\";\n",
        version, timestamp, commit);
    f.write_all(contents.as_bytes()).unwrap();
}

fn get_head_commit() -> String {
    let output = Command::new("git").args(&["rev-parse", "--short", "HEAD"]).output().unwrap();
    let mut rev = String::from_utf8(output.stdout).unwrap();
    rev.pop();
    rev
}
