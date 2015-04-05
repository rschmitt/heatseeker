extern crate time;
extern crate git2;

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use time::{strftime, now};
use git2::Repository;

fn main() {
    let basedir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let version = env!("CARGO_PKG_VERSION");
    let dest_path = Path::new(&basedir).join("src/version.rs");
    let mut f = File::create(&dest_path).unwrap();

    let timestamp = strftime("%F %H:%M:%S %z", &now()).unwrap();
    let commit = get_head_commit(&basedir);

    let contents = format!(
"pub const VERSION: &'static str = \"{}\";
pub const TIMESTAMP: &'static str = \"{}\";
pub const COMMIT: &'static str = \"{}\";\n",
        version, timestamp, commit);
    f.write_all(contents.as_bytes()).unwrap();
}

fn get_head_commit(pwd: &str) -> String {
    let repo = Repository::open(pwd).unwrap();
    let revspec = repo.revparse(&"HEAD").unwrap();
    let commit_id: git2::Oid = revspec.from().unwrap().id();
    let full_id = commit_id.to_string();
    let short_id = &full_id[0..8];
    short_id.to_string()
}
