#![cfg_attr(test, allow(dead_code))]
#![feature(collections, os, io, std_misc, libc)]
#![cfg_attr(not(windows), feature(path))]

extern crate libc;

mod args;
mod matching;
mod screen;
#[cfg(not(windows))] mod ansi;

use std::os;
use std::old_io;
use std::cmp::min;
use screen::Screen;
use screen::Key;
use screen::Key::*;
use self::SearchState::*;

fn main() {
    let args = match args::parse_args() {
        Some(args) => args,
        None => {
            os::set_exit_status(1);
            return;
        },
    };

    if args.help { return; }

    let choices = read_choices();
    let initial_search = args.initial_search.clone();
    let choices = choices.iter().map(|x| &x[]).collect::<Vec<&str>>();
    if args.use_first {
        let matches = matching::compute_matches(&choices, &initial_search);
        println!("{}", matches.get(0).unwrap_or(&""));
        return;
    } else {
        event_loop(&choices, &initial_search);
    }
}

fn event_loop(choices: &[&str], initial_search: &str) {
    let mut search = Search::new(choices, initial_search.to_string());
    let mut screen = Screen::open_screen();

    loop {
        search.recompute_matches();

        match search.state {
            InProgress => draw_screen(&mut screen, &search),
            _ => break,
        }

        let keys = screen.get_buffered_keys();
        for key in keys.iter() {
            handle_key(&mut search, key, screen.visible_choices);
        }
    }

    screen.move_cursor_to_bottom();
    println!("{}", search.get_selection());
}

fn handle_key(search: &mut Search, key: &Key, visible_choices: u16) {
    match *key {
        Char(x) => search.append(x),
        Backspace => search.backspace(),
        Control('h') => search.backspace(),
        Control('w') => search.delete_word(),
        Control('u') => search.clear_query(),
        Control('c') => search.cancel(),
        Control('g') => search.cancel(),
        Control('n') => search.down(visible_choices),
        Control('p') => search.up(),
        Enter => search.done(),
        _ => {}
    }
}

struct Search<'a> {
    choices: &'a [&'a str],
    query: String,
    matches: Vec<&'a str>,
    stale: bool,
    index: usize,
    state: SearchState,
}

#[derive(PartialEq, Eq)]
enum SearchState {
    InProgress,
    Done,
    Canceled,
}

impl<'a> Search<'a> {
    fn new(choices: &'a [&'a str], initial_search: String) -> Search<'a> {
        let matches = choices.to_vec();
        Search {
            choices: choices,
            query: initial_search,
            matches: matches,
            stale: true,
            index: 0,
            state: InProgress,
        }
    }

    fn up(&mut self) {
        self.index = if self.index == 0 { 0 } else { self.index - 1 };
    }

    fn down(&mut self, visible_choices: u16) {
        let limit = min(visible_choices as usize - 1, self.matches.len() - 1);
        self.index = min(self.index + 1, limit);
    }

    fn backspace(&mut self) {
        self.query.pop();
        self.stale = true;
        self.index = 0;
        self.matches = self.choices.to_vec();
    }

    fn delete_word(&mut self) {
        self.stale = true;
        delete_last_word(&mut self.query);
        self.matches = self.choices.to_vec();
    }

    fn append(&mut self, c: char) {
        self.query.push(c);
        self.stale = true;
        self.index = 0;
    }

    fn clear_query(&mut self) {
        self.query.clear();
        self.matches = self.choices.to_vec();
    }

    fn recompute_matches(&mut self) {
        if self.stale {
            self.matches = matching::compute_matches(&self.matches, &self.query);
            self.stale = false;
        }
    }

    fn get_selection(&mut self) -> String {
        if self.state == Canceled {
            "".to_string()
        } else {
            self.recompute_matches();
            self.matches.get(self.index).unwrap_or(&"").to_string()
        }
    }

    fn cancel(&mut self) {
        self.state = Canceled
    }

    fn done(&mut self) {
        self.state = Done
    }
}

fn draw_screen(screen: &mut Screen, search: &Search) {
    screen.hide_cursor();
    screen.blank_screen();
    screen.write(&format!("> {} ({} choices)\n", search.query, search.choices.len()));

    print_matches(screen, &search.matches, search.index);

    let start = screen.start_line;
    screen.move_cursor(start, 2 + search.query.len() as u16);
    screen.show_cursor();
}

fn print_matches(screen: &mut Screen, matches: &Vec<&str>, index: usize) {
    let mut i = 1;
    for choice in matches.iter() {
        if i == index + 1 {
            screen.write_inverted(&choice);
        } else {
            screen.write(&choice);
        }
        if i >= screen.visible_choices as usize {
            return;
        } else {
            screen.write("\n");
        }
        i += 1;
    }
}

fn read_choices() -> Vec<String> {
    let mut stdin = old_io::stdio::stdin();
    let mut lines = Vec::new();

    while let Ok(mut s) = stdin.read_line() {
        trim(&mut s);
        lines.push(s);
    }

    lines
}

pub fn trim(s: &mut String) {
    while let Some(x) = s.pop() {
        if x != '\n' {
            s.push(x);
            return;
        }
    }
}

fn delete_last_word(s: &mut String) {
    let mut deleted_something = false;
    while let Some(x) = s.pop() {
        if x == ' ' {
            if deleted_something {
                s.push(x);
                return;
            }
        } else {
            deleted_something = true;
        }
    }
}

#[test]
fn trim_test() {
    fn should_become(before: &str, after: &str) {
        let mut x = before.to_string();
        trim(&mut x);
        assert_eq!(after.to_string(), x);
    }
    should_become("", "");
    should_become("\n", "");
    should_become("\n\n", "");
    should_become("asdf", "asdf");
    should_become("asdf\n", "asdf");
    should_become("asdf\nasdf\n", "asdf\nasdf");
}

#[test]
fn delete_word_test() {
    fn should_become(before: &str, after: &str) {
        let mut x = before.to_string();
        delete_last_word(&mut x);
        assert_eq!(after.to_string(), x);
    }
    should_become("", "");
    should_become("a", "");
    should_become("asdf", "");
    should_become("asdf asdf asdf", "asdf asdf ");
    should_become("asdf asdf asdf ", "asdf asdf ");
    should_become("asdf asdf asdf  ", "asdf asdf ");
}
