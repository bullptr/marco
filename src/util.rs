use shell_words;
use similar::{ChangeTag, TextDiff};

/// Parses a commandline string into a program and its arguments
#[allow(unused)]
pub fn parse_shell_cmd(cmd: &str) -> Option<(String, Vec<String>)> {
    match shell_words::split(cmd) {
        Ok(words) if !words.is_empty() => {
            let prog = words[0].clone();
            let args = words[1..].to_vec();
            Some((prog, args))
        }
        _ => None,
    }
}

/// Checks if a &str is probably JSON (by looking for `{` or `[`)
pub fn is_json(s: &str) -> bool {
    let s = s.trim();
    s.starts_with('{') || s.starts_with('[')
}

/// Fast JSON normalization and comparison. Returns true if parsed JSONs are equal.
pub fn normalized_json_eq(a: &str, b: &str) -> bool {
    let v1: Result<serde_json::Value, _> = serde_json::from_str(a);
    let v2: Result<serde_json::Value, _> = serde_json::from_str(b);
    match (v1, v2) {
        (Ok(j1), Ok(j2)) => j1 == j2,
        _ => false,
    }
}

/// Pretty print text diff
pub fn print_diff(actual: &str, expected: &str) {
    let diff = TextDiff::from_lines(actual.trim(), expected.trim());
    for change in diff.iter_all_changes() {
        let (tag_symbol, color) = match change.tag() {
            ChangeTag::Delete => ("\x1b[91m-\x1b[0m ", "\x1b[97m"),
            ChangeTag::Insert => ("\x1b[92m+\x1b[0m ", "\x1b[97m"),
            ChangeTag::Equal => ("  ", "\x1b[90m"),
        };
        print!("    {}{}{}\x1b[0m", color, tag_symbol, change);
    }
}
