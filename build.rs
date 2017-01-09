extern crate time;

use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use time::{strftime, now};
use std::process::Command;

fn main() {
    let version = env::var("CARGO_PKG_VERSION").unwrap();
    let timestamp = strftime("%F %H:%M:%S %z", &now()).unwrap();
    let commit = get_head_commit().unwrap_or("".to_string());
    let target = env::var("TARGET").unwrap();

    write(&version, "version.txt");
    write(&timestamp, "timestamp.txt");
    write(&target, "target.txt");
    write(&commit, "commit.txt");
}

fn get_head_commit() -> Result<String, Box<Error>> {
    let output = Command::new("git").args(&["rev-parse", "--short", "HEAD"]).output()?;
    let mut rev = String::from_utf8(output.stdout)?;
    rev.pop();
    Ok(rev)
}

fn write(contents: &str, filename: &str) {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let file = out_dir.join(filename);
    let debug_path = file.to_str().unwrap().to_string();
    File::create(file)
        .expect(&format!("Unable to create file {:?}", debug_path))
        .write_all(contents.as_bytes())
        .expect(&format!("Unable to write string '{}' to file {:?}", contents, debug_path));
}
