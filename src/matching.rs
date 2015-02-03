use std::ascii::AsciiExt;
use std::cmp::min;

pub fn compute_matches<'a>(choices: &'a Vec<String>, query: &str) -> Vec<&'a String> {
    struct ScoredChoice<'a> {
        score: f64,
        choice: &'a String,
    };
    let mut ret = Vec::new();
    for choice in choices.iter() {
        let score = score(&choice, query);
        if score > 0_f64 {
            ret.push(ScoredChoice{ score: score, choice: choice });
        }
    }
    ret.sort_by(|x, y| y.score.partial_cmp(&x.score).unwrap());
    ret.iter().map(|s| s.choice).collect()
}

#[allow(dead_code)]
fn get_slice_indices(length: usize, workers: usize, idx: usize) -> (usize, usize) {
    let lb = (length as f64 / workers as f64) * idx as f64;
    let ub = (length as f64 / workers as f64) * (idx + 1) as f64;
    (lb as usize, ub as usize)
}

#[test]
fn get_slice_indices_test() {
    assert_eq!(get_slice_indices(100, 1, 0), (0, 100));

    assert_eq!(get_slice_indices(100, 2, 1), (50, 100));
    assert_eq!(get_slice_indices(100, 2, 0), (0, 50));

    assert_eq!(get_slice_indices(100, 3, 0), (0, 33));
    assert_eq!(get_slice_indices(100, 3, 1), (33, 66));
    assert_eq!(get_slice_indices(100, 3, 2), (66, 100));

    assert_eq!(get_slice_indices(100, 4, 0), (0, 25));
    assert_eq!(get_slice_indices(100, 4, 1), (25, 50));
    assert_eq!(get_slice_indices(100, 4, 2), (50, 75));
    assert_eq!(get_slice_indices(100, 4, 3), (75, 100));

    assert_eq!(get_slice_indices(12, 12, 11), (11, 12));
    assert_eq!(get_slice_indices(12, 12, 0), (0, 1));

    assert_eq!(get_slice_indices(1, 2, 0), (0, 0));
    assert_eq!(get_slice_indices(1, 2, 1), (0, 1));
}

fn score(choice: &str, query: &str) -> f64 {
    if query.len() == 0 {
        return 1.0;
    }
    if choice.len() == 0 {
        return 0.0;
    }

    let query = query.to_ascii_lowercase();
    let choice = choice.to_ascii_lowercase();

    match compute_match_length(&choice, &query) {
        None => return 0.0,
        Some(match_length) => {
            let score = query.len() as f64 / match_length as f64;
            return score as f64 / choice.len() as f64;
        }
    }
}

fn compute_match_length(string: &str, chars: &str) -> Option<usize> {
    let first_char = chars.char_at(0);
    let rest = &chars[1..chars.len()];
    let indices = find_char_in_string(string, first_char);

    let mut current_min = None;
    for i in indices.iter() {
        let last_index = find_end_of_match(string, rest, *i);
        if last_index.is_some() {
            let idx = last_index.unwrap() - *i + 1;
            if current_min.is_some() {
                let cm = min(current_min.unwrap(), idx);
                current_min = Some(cm);
            } else {
                current_min = Some(idx)
            }
        }
    }
    return current_min;
}

fn find_char_in_string(string: &str, char: char) -> Vec<usize> {
    let mut indices = Vec::new();
    let mut i = 0;
    for c in string.chars() {
        if c == char {
            indices.push(i);
        }
        i = i + 1;
    }
    return indices;
}

fn find_end_of_match(string: &str, rest_of_query: &str, first_index: usize) -> Option<usize> {
    let mut last_index = first_index + 1;
    let chars_in_string = string.chars().collect::<Vec<_>>().len();
    for c in rest_of_query.chars() {
        let current_substring = string.slice_chars(last_index, chars_in_string);
        match current_substring.find(c) {
            None => return None,
            Some(ref i) => {
                last_index += *i + 1;
            }
        }
    }
    return Some(last_index - 1);
}

#[test]
fn find_end_of_match_test() {
    assert_eq!(find_end_of_match("a", "a", 0), None);
    assert_eq!(find_end_of_match("ba", "a", 1), None);
    assert_eq!(find_end_of_match("aaa", "aa", 0), Some(2));
    assert_eq!(find_end_of_match("aaa", "b", 0), None);
    assert_eq!(find_end_of_match("this is a long match", "this is a match", 0), None);
    assert_eq!(find_end_of_match("this is a long match", "his is a match", 0), Some(19));
    assert_eq!(find_end_of_match("./rust/x86_64-apple-darwin/test/run-pass/process-spawn-with-unicode-params-πЯ音æ∞/child.stage2-x86_64-apple-darwin", "ust", 2), Some(5));
}

#[test]
fn basic_scoring() {
    assert_eq!(score("", "a"), 0.0);
    assert_eq!(score("a", ""), 1.0);
    assert_eq!(score("short", "longer"), 0.0);
    assert_eq!(score("a", "b"), 0.0);
    assert_eq!(score("ab", "ac"), 0.0);

    assert!(score("a", "a") > 0.0);
    assert!(score("ab", "a") > 0.0);
    assert!(score("ba", "a") > 0.0);
    assert!(score("bab", "a") > 0.0);
    assert!(score("babababab", "aaaa") > 0.0);

    assert_eq!(score("a", "a"), 1_f64 / "a".len() as f64);
    assert_eq!(score("ab", "ab"), 0.5);
    assert_eq!(score("a long string", "a long string"), 1_f64 / "a long string".len() as f64);
    assert_eq!(score("spec/search_spec.rb", "sear"), 1_f64 / "spec/search_spec.rb".len() as f64);
}

#[test]
fn character_matching() {
    assert!(score("/! symbols $^", "/!$^") > 0.0);

    assert_eq!(score("a", "A"), 1.0);
    assert_eq!(score("A", "a"), 1.0);

    assert_eq!(score("a", "aa"), 0.0);
}

#[test]
fn match_equality() {
    assert!(score("selecta.gemspec", "asp") > score("algorithm4_spec.rb", "asp"));
    assert!(score("README.md", "em") > score("benchmark.rb", "em"));
    assert!(score("search.rb", "sear") > score("spec/search_spec.rb", "sear"));

    assert!(score("fbb", "fbb") > score("foo bar baz", "fbb"));
    assert!(score("foo", "foo") > score("longer foo", "foo"));
    assert!(score("foo", "foo") > score("foo longer", "foo"));
    assert!(score("1/2/3/4", "1/2/3") > score("1/9/2/3/4", "1/2/3"));

    assert!(score("long 12 long", "12") > score("1 long 2", "12"));

    assert_eq!(score("121padding2", "12"), 1.0 / "121padding2".len() as f64);
    assert_eq!(score("1padding212", "12"), 1.0 / "1padding212".len() as f64);
}
