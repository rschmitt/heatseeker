#![cfg_attr(test, allow(dead_code))]
#![cfg_attr(nightly, feature(scoped))]

extern crate unicode_width;

mod args;
mod matching;
mod screen;
mod version;
#[cfg(not(windows))] mod ansi;

use version::*;
use std::collections::HashSet;
use std::process;
use std::io::{stdin, BufRead};
use std::cmp::min;
use screen::Screen;
use screen::Key;
use screen::Key::*;
use self::SearchState::*;
use unicode_width::UnicodeWidthStr;

#[cfg(windows)] const NEWLINE: &'static str = "\r\n";
#[cfg(not(windows))] const NEWLINE: &'static str = "\n";

fn main() {
    let args = match args::parse_args() {
        Some(args) => args,
        None => {
            process::exit(1);
        },
    };

    if args.help { return; }

    if args.version {
        if COMMIT == "" {
            println!("heatseeker {} (built {})", VERSION, TIMESTAMP);
        } else {
            println!("heatseeker {} ({}) (built {})", VERSION, COMMIT, TIMESTAMP);
        }
        return;
    }

    let choices = read_choices();
    let initial_search = args.initial_search.clone();
    let choices = choices.iter().map(|x| &x[..]).collect::<Vec<&str>>();
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

    if cfg!(windows) {
        screen.blank_screen();
    } else {
        screen.move_cursor_to_bottom();
    }
    print!("{}", search.get_selections());
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
        Control('p') => search.up(visible_choices),
        Control('t') => { search.toggle_selection(); search.down(visible_choices); },
        Control('n') => search.down(visible_choices),
        Tab => search.down(visible_choices),
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
    selections: HashSet<String>,
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
            selections: HashSet::new(),
        }
    }

    fn up(&mut self, visible_choices: u16) {
        let limit = min(visible_choices as usize - 1, self.matches.len() - 1);
        self.index = if self.index == 0 { limit } else { self.index - 1 };
    }

    fn down(&mut self, visible_choices: u16) {
        let limit = min(visible_choices as usize - 1, self.matches.len() - 1);
        self.index = if self.index == limit { 0 } else { self.index + 1 }
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

    fn toggle_selection(&mut self) {
        self.recompute_matches();
        let selection = self.matches.get(self.index).unwrap_or(&"").to_string();
        if self.selections.contains(&selection) {
            self.selections.remove(&selection);
        } else {
            self.selections.insert(selection);
        }
    }

    fn get_selections(&mut self) -> String {
        let mut ret = String::new();
        if self.state != Canceled {
            self.recompute_matches();
            let selection = self.matches.get(self.index).unwrap_or(&"").to_string();
            self.selections.insert(selection);
        }
        for selection in self.selections.iter() {
            ret.push_str(selection);
            ret.push_str(NEWLINE);
        }
        ret
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
    screen.write(&format!("> {} ({}/{} choices){}", search.query, search.matches.len(), search.choices.len(), NEWLINE));

    print_matches(screen, &search.matches, &search.query, search.index, &search.selections);

    let query_str: &str = &search.query;
    screen.move_cursor_to_prompt_line(2 + UnicodeWidthStr::width(query_str) as u16);
    screen.show_cursor();
}

fn print_matches(screen: &mut Screen, matches: &[&str], query: &str, index: usize, selections: &HashSet<String>) {
    let mut i = 1;
    for choice in matches.iter() {
        let indices = matching::visual_score(choice, query);
        let max_width = screen.width;
        let mut annotated_choice = choice.to_string();
        if selections.contains(&annotated_choice) {
            annotated_choice.push(' ');
            if cfg!(windows) {
                annotated_choice.push('√');
            } else {
                annotated_choice.push('✓');
            }
        }
        print_match(&annotated_choice, &indices, max_width, &mut |s, highlight| {
            if i == index + 1 {
                if highlight { screen.write_red_inverted(s); } else { screen.write_inverted(s); }
            } else {
                if highlight { screen.write_red(s); } else { screen.write(s); }
            }
        });
        if i >= screen.visible_choices as usize {
            return;
        } else {
            screen.write(NEWLINE);
        }
        i += 1;
    }
}

fn print_match(choice: &str, indices: &[usize], max_width: u16, writer: &mut FnMut(&str, bool)) {
    #[cfg(windows)] const MARGIN: u16 = 1;
    #[cfg(not(windows))] const MARGIN: u16 = 0;
    let max_width = max_width - MARGIN;
    let chars_in_choice = choice.chars().count();
    let chars_to_draw = min(chars_in_choice, max_width as usize);
    let mut last_idx = 0;
    for &idx in indices {
        let idx = min(idx, chars_to_draw);
        if last_idx >= chars_to_draw {
            return;
        }
        writer(&slice_chars(choice, last_idx, idx), false);
        if idx == chars_to_draw { return }
        writer(&slice_chars(choice, idx, idx + 1), true);
        last_idx = idx + 1;
    }
    writer(&slice_chars(choice, last_idx, chars_to_draw), false);
}

fn read_choices() -> Vec<String> {
    let stdin = stdin();
    let mut lines = Vec::new();

    let mut stdin = stdin.lock();
    loop {
        let mut s = String::new();
        stdin.read_line(&mut s).unwrap();
        if s.len() == 0 {
            break;
        } else {
            trim(&mut s);
            lines.push(s);
        }
    }

    lines
}

pub fn trim(s: &mut String) {
    while let Some(x) = s.pop() {
        if x != '\n' && x != '\r' {
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

// This is technically a standard library function, but it's plastered with
// stability warnings and therefore only available on the nightly channel.
fn slice_chars(s: &str, begin: usize, end: usize) -> &str {
    assert!(begin <= end);
    let mut count = 0;
    let mut begin_byte = None;
    let mut end_byte = None;

    // This could be even more efficient by not decoding,
    // only finding the char boundaries
    for (idx, _) in s.char_indices() {
        if count == begin { begin_byte = Some(idx); }
        if count == end { end_byte = Some(idx); break; }
        count += 1;
    }
    if begin_byte.is_none() && count == begin { begin_byte = Some(s.len()) }
    if end_byte.is_none() && count == end { end_byte = Some(s.len()) }

    match (begin_byte, end_byte) {
        (None, _) => panic!("slice_chars: `begin` is beyond end of string"),
        (_, None) => panic!("slice_chars: `end` is beyond end of string"),
        (Some(a), Some(b)) => unsafe { s.slice_unchecked(a, b) }
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
    should_become("asdf\r\n", "asdf");
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
