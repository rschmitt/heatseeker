use std::ascii::AsciiExt;

macro_rules! chars {
    ($str:expr) => (
        &$str.chars().collect::<Vec<char>>()
    );
}

pub fn compute_matches<'a>(choices: &[&'a str], query: &str) -> Vec<&'a str> {
    struct ScoredChoice<'a> {
        score: f64,
        choice: &'a str,
    };
    let mut ret = Vec::new();
    for choice in choices.iter() {
        let score = score(&choice, query);
        if score > 0_f64 {
            ret.push(ScoredChoice{ score: score, choice: &choice });
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
    let query = chars!(query);
    let choice = chars!(choice);

    match compute_match_length(choice, query) {
        None => return 0.0,
        Some(match_length) => {
            let score = query.len() as f64 / match_length as f64;
            return score as f64 / choice.len() as f64;
        }
    }
}

fn compute_match_length(string: &[char], query: &[char]) -> Option<usize> {
    get_match_length(get_longest_match(string, query))
}

fn get_longest_match(string: &[char], query: &[char]) -> Option<(usize, usize)> {
    let first_char = query[0];
    let rest = &query[1..];
    let indices = find_char_in_string(string, first_char);

    let mut current_bounds: Option<(usize, usize)> = None;
    let smallest_possible_match = query.len();
    for &i in &indices {
        if let Some(last_index) = find_end_of_match(string, rest, i) {
            let last_bounds = Some((i, last_index));
            let last_match_len = get_match_length(last_bounds).unwrap();
            if current_bounds.is_none() || last_match_len < get_match_length(current_bounds).unwrap() {
                current_bounds = last_bounds;
                if last_match_len == smallest_possible_match {
                    break;
                }
            }
        }
    }
    current_bounds
}

fn get_match_length(bounds: Option<(usize, usize)>) -> Option<usize> {
    if let Some((lb, ub)) = bounds {
        Some(ub - lb + 1)
    } else {
        None
    }
}

fn find_char_in_string(string: &[char], char: char) -> Vec<usize> {
    let mut indices = Vec::new();
    for (i, c) in string.iter().enumerate() {
        if char == *c {
            indices.push(i);
        }
    }
    return indices;
}

fn find_end_of_match(string: &[char], rest_of_query: &[char], first_index: usize) -> Option<usize> {
    if let Some(indices) = get_match_indices(string, rest_of_query, first_index) {
        Some(indices[indices.len() - 1])
    } else {
        None
    }
}

fn get_match_indices(string: &[char], rest_of_query: &[char], first_index: usize) -> Option<Vec<usize>> {
    let mut ret = Vec::new();
    let mut last_index = first_index + 1;
    ret.push(first_index);
    for c in rest_of_query.iter() {
        let current_substring = &string[last_index..];
        let mut index = None;
        for (i, x) in current_substring.iter().enumerate() {
            if x == c {
                index = Some(i);
                break;
            }
        }
        if index.is_some() {
            last_index += index.unwrap() + 1;
            ret.push(last_index - 1);
        } else {
            return None;
        }
    }
    Some(ret)
}

#[test]
fn get_match_indices_test() {
    assert_eq!(get_match_indices(chars!("asdf"), chars!("sdf"), 0).unwrap(), vec![0, 1, 2, 3]);
    assert_eq!(get_match_indices(chars!("aoeuasdf"), chars!("sdf"), 4).unwrap(), vec![4, 5, 6, 7]);
    assert_eq!(get_match_indices(chars!(" a s d f"), chars!("sdf"), 1).unwrap(), vec![1, 3, 5, 7]);
}

#[test]
fn find_end_of_match_test() {
    assert_eq!(find_end_of_match(chars!("a"), chars!("a"), 0), None);
    assert_eq!(find_end_of_match(chars!("ba"), chars!("a"), 1), None);
    assert_eq!(find_end_of_match(chars!("aaa"), chars!("aa"), 0), Some(2));
    assert_eq!(find_end_of_match(chars!("aaa"), chars!("b"), 0), None);
    assert_eq!(find_end_of_match(chars!("this is a long match"), chars!("this is a match"), 0), None);
    assert_eq!(find_end_of_match(chars!("this is a long match"), chars!("his is a match"), 0), Some(19));
    assert_eq!(find_end_of_match(chars!("./rust/x86_64-apple-darwin/test/run-pass/process-spawn-with-unicode-params-πЯ音æ∞/child.stage2-x86_64-apple-darwin"), chars!("ust"), 2), Some(5));
}

#[test]
fn unicode_boundary_handling_test() {
    score("./rust/x86_64-apple-darwin/test/run-pass/process-spawn-with-unicode-params-πЯ音æ∞/child.stage2-x86_64-apple-darwin", "he");
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
