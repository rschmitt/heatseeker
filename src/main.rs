mod args;
mod matching;
mod screen;
#[cfg(not(windows))] mod ansi;

use std::cmp::min;
use std::collections::HashSet;
use std::env;
use std::io::{stdin, BufRead};
use std::process;
use screen::Screen;
use screen::Key;
use screen::Key::*;
use self::SearchState::*;
use unicode_width::UnicodeWidthStr;

#[cfg(windows)] pub const NEWLINE: &'static str = "\r\n";
#[cfg(not(windows))] pub const NEWLINE: &'static str = "\n";

const VERSION: &'static str = include_str!(concat!(env!("OUT_DIR"), "/version.txt"));
const TIMESTAMP: &'static str = include_str!(concat!(env!("OUT_DIR"), "/timestamp.txt"));
const TARGET: &'static str = include_str!(concat!(env!("OUT_DIR"), "/target.txt"));
const COMMIT: &'static str = include_str!(concat!(env!("OUT_DIR"), "/commit.txt"));

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    let args = match args::parse_args() {
        Some(args) => args,
        None => {
            process::exit(1);
        },
    };

    if args.help { return; }

    if args.version {
        if COMMIT == "" {
            println!("heatseeker {} (built {} for {})", VERSION, TIMESTAMP, TARGET);
        } else {
            println!("heatseeker {}-{} (built {} for {})", VERSION, COMMIT, TIMESTAMP, TARGET);
        }
        return;
    }

    let choices = read_choices();
    let initial_search = args.initial_search.clone();
    let choices = choices.iter().map(|x| &x[..]).collect::<Vec<&str>>();
    if args.use_first {
        let matches = matching::compute_matches(&choices, &initial_search, args.filter_only);
        println!("{}", matches.get(0).unwrap_or(&""));
        return;
    } else {
        let desired_rows = if args.full_screen { 999 } else { 20 };
        let selections = event_loop(desired_rows, &choices, &initial_search, args.filter_only);
        print!("{}", selections);
    }
}

fn event_loop(desired_rows: u16, choices: &[&str], initial_search: &str, filter_only: bool) -> String {
    let mut search = Search::new(choices, initial_search.to_string(), filter_only);
    let mut screen = screen::new(desired_rows);

    loop {
        search.recompute_matches();

        match search.state {
            InProgress => draw_screen(screen.as_mut(), &search),
            _ => break,
        }

        let keys = screen.get_buffered_keys();
        for key in &keys {
            handle_key(&mut search, key, screen.visible_choices());
        }
    }

    screen.blank_screen();
    search.get_selections()
}

fn handle_key(search: &mut Search, key: &Key, visible_choices: u16) {
    match *key {
        Char(x) => search.append(x),
        Backspace |
        Control('h') => search.backspace(),
        Control('w') => search.delete_word(),
        Control('u') => search.clear_query(),
        Control('r') => std::panic!("This is a test backtrace"),
        Control('c') |
        Control('g') => search.cancel(),
        Control('t') => { search.toggle_selection(); search.down(visible_choices); },
        Control('p') |
        Up |
        ShiftTab => search.up(visible_choices),
        Control('n') |
        Down |
        Tab => search.down(visible_choices),
        Home => search.home(),
        End => search.end(visible_choices),
        Enter => search.done(),
        Control('b') |
        PgUp => search.pgup(visible_choices),
        Control('f') |
        PgDown => search.pgdown(visible_choices),
        _ => {},
    }
}

struct Search<'a> {
    choices: &'a [&'a str],
    query: String,
    matches: Vec<&'a str>,
    stale: bool,
    scroll_offset: usize,
    cursor_index: usize,
    state: SearchState,
    selections: HashSet<String>,
    filter_only: bool,
}

#[derive(PartialEq, Eq)]
enum SearchState {
    InProgress,
    Done,
    Canceled,
}

impl<'a> Search<'a> {
    fn new(choices: &'a [&'a str], initial_search: String, filter_only: bool) -> Search<'a> {
        let matches = choices.to_vec();
        Search {
            choices,
            query: initial_search,
            matches,
            stale: true,
            scroll_offset: 0,
            cursor_index: 0,
            state: InProgress,
            selections: HashSet::new(),
            filter_only,
        }
    }

    fn up(&mut self, visible_choices: u16) {
        let match_count = self.matches.len();
        let limit = min(visible_choices as usize - 1, match_count - 1);
        let should_wrap = self.scroll_offset == 0;
        if self.cursor_index == 0 {
            if should_wrap {
                if match_count > visible_choices as usize {
                    self.scroll_offset = match_count - visible_choices as usize;
                    self.cursor_index = visible_choices as usize - 1;
                } else {
                    self.cursor_index = limit;
                }
            } else {
                self.scroll_offset -= 1;
            }
        } else {
            self.cursor_index -= 1;
        }
    }

    fn down(&mut self, visible_choices: u16) {
        let match_count = self.matches.len();
        let limit = min(visible_choices as usize - 1, match_count - 1);
        let should_wrap = self.cursor_index + self.scroll_offset == match_count - 1;
        if self.cursor_index == limit {
            if should_wrap {
                self.cursor_index = 0;
                self.scroll_offset = 0;
            } else {
                self.scroll_offset += 1;
            }
        } else {
            self.cursor_index += 1;
        }
    }

    fn home(&mut self) {
        self.cursor_index = 0;
        self.scroll_offset = 0;
    }

    fn end(&mut self, visible_choices: u16) {
        self.home();
        self.up(visible_choices);
    }

    fn pgup(&mut self, visible_choices: u16) {
        for _ in 0..visible_choices {
            if self.scroll_offset == 0 && self.cursor_index == 0 {
                return;
            }
            self.up(visible_choices);
        }
    }

    fn pgdown(&mut self, visible_choices: u16) {
        for _ in 0..visible_choices {
            if self.scroll_offset + self.cursor_index == self.matches.len() - 1 {
                return;
            }
            self.down(visible_choices);
        }
    }

    fn backspace(&mut self) {
        self.query.pop();
        self.stale = true;
        self.cursor_index = 0;
        self.scroll_offset = 0;
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
        self.cursor_index = 0;
        self.scroll_offset = 0;
    }

    fn clear_query(&mut self) {
        self.query.clear();
        self.cursor_index = 0;
        self.scroll_offset = 0;
        self.matches = self.choices.to_vec();
    }

    fn recompute_matches(&mut self) {
        if self.stale {
            self.matches = matching::compute_matches(&self.matches, &self.query, self.filter_only);
            self.stale = false;
        }
    }

    fn toggle_selection(&mut self) {
        self.recompute_matches();
        let selection = self.matches.get(self.scroll_offset + self.cursor_index).unwrap_or(&"").to_string();
        if self.selections.contains(&selection) {
            self.selections.remove(&selection);
        } else {
            self.selections.insert(selection);
        }
    }

    fn get_selections(&mut self) -> String {
        let mut ret = String::new();
        if self.state != Canceled {
            for selection in &self.selections {
                ret.push_str(selection);
                ret.push_str(NEWLINE);
            }
            if ret.is_empty() {
                self.recompute_matches();
                let selection = self.matches.get(self.scroll_offset + self.cursor_index).unwrap_or(&"").to_string();
                ret.push_str(&selection);
            }
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

fn draw_screen(screen: &mut dyn Screen, search: &Search) {
    screen.hide_cursor();
    screen.blank_screen();
    screen.write(&format!("> {} ({}/{} choices){}", search.query, search.matches.len(), search.choices.len(), NEWLINE));

    print_matches(screen, &search.matches, &search.query, search.scroll_offset, search.cursor_index, &search.selections);

    let query_str: &str = &search.query;
    screen.move_cursor_to_prompt_line(2 + UnicodeWidthStr::width(query_str) as u16);
    screen.show_cursor();
}

fn print_matches(screen: &mut dyn Screen, matches: &[&str], query: &str, scroll_offset: usize, cursor_index: usize, selections: &HashSet<String>) {
    let mut i = 1;
    for choice in matches[scroll_offset..].iter() {
        let indices = matching::visual_score(choice, query);
        let max_width = screen.width();
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
            if i == cursor_index + 1 {
                if highlight { screen.write_red_inverted(s); } else { screen.write_inverted(s); }
            } else if highlight {
                screen.write_red(s);
            } else { screen.write(s); }
        });
        if i >= screen.visible_choices() as usize {
            return;
        }
        screen.write(NEWLINE);
        i += 1;
    }
}

fn print_match(choice: &str, indices: &[usize], max_width: u16, writer: &mut dyn FnMut(&str, bool)) {
    #[cfg(windows)] const MARGIN: u16 = 1;
    #[cfg(not(windows))] const MARGIN: u16 = 0;
    let max_width = max_width - MARGIN;
    let chars_in_choice = choice.chars().count();
    let mut chars_to_draw = min(chars_in_choice, max_width as usize);
    while UnicodeWidthStr::width(slice_chars(choice, 0, chars_to_draw)) > max_width as usize {
        chars_to_draw -= 1;
    }
    let mut last_idx = 0;
    for &idx in indices {
        let idx = min(idx, chars_to_draw);
        if last_idx >= chars_to_draw {
            return;
        }
        writer(slice_chars(choice, last_idx, idx), false);
        if idx == chars_to_draw { return }
        writer(slice_chars(choice, idx, idx + 1), true);
        last_idx = idx + 1;
    }
    writer(slice_chars(choice, last_idx, chars_to_draw), false);
}

fn read_choices() -> Vec<String> {
    let stdin = stdin();
    let mut lines = Vec::new();

    let mut stdin = stdin.lock();
    let mut first_error = None;
    let mut suppressed = 0;
    loop {
        let mut s = String::new();
        match stdin.read_line(&mut s) {
            Ok(_) => {
                if s.is_empty() { break; }
                trim(&mut s);
                lines.push(s);
            },
            Err(e) => {
                if first_error.is_some() {
                    suppressed = suppressed + 1;
                } else {
                    first_error = Some(e);
                }
            }
        }
    }
    if first_error.is_some() {
        eprintln!("Warning: Failed to parse one or more lines (\"{}\"); {} additional error(s) suppressed", first_error.unwrap(), suppressed);
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
        (Some(a), Some(b)) => unsafe { s.get_unchecked(a..b) }
    }
}

#[cfg(test)]
mod tests {
    use super::{trim, delete_last_word};

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
}
