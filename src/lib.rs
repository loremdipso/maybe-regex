use crate::utils::{remove_first_n_chars, remove_last_n_chars};
use lazy_static::lazy_static;
use log::error;
use regex::{Captures, Regex, Replacer};
use std::cmp::Ordering;

mod utils;

lazy_static! {
    // Simplistic check to see if a string is likely a regex.
    // TODO: is there a way to make this actually correct?
    static ref REGEX_REGEX: Regex = Regex::new(r"[\\b\$\^\[\]\+\*\.]").unwrap();
}

#[derive(Debug, Clone)]
pub struct MaybeRegex {
    data: TagWrapperData,
    original: String,
    pub is_negative: bool,
    case_sensitive: bool,
}

impl PartialEq for MaybeRegex {
    fn eq(&self, other: &Self) -> bool {
        self.original == other.original && self.is_negative == other.is_negative
    }
}

impl PartialOrd for MaybeRegex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (&self.original, self.is_negative).partial_cmp(&(&other.original, other.is_negative))
    }
}

#[derive(Debug, Clone)]
pub enum TagWrapperData {
    Raw(String),
    Regex(Regex),
}

impl MaybeRegex {
    pub fn new<S: AsRef<str>>(s: S) -> Self {
        Self::from(s)
    }

    pub fn from<S: AsRef<str>>(s: S) -> Self {
        let s = s.as_ref();
        let (s, is_negative) = if s.starts_with("-") {
            (remove_first_n_chars(s, 1), true)
        } else if s.ends_with("-") {
            (remove_last_n_chars(s, 1), true)
        } else {
            (s.into(), false)
        };

        match get_regex(&s) {
            Some(regex) => Self {
                data: TagWrapperData::Regex(regex),
                original: s,
                is_negative,
                case_sensitive: false,
            },
            None => Self {
                data: TagWrapperData::Raw(s.clone()),
                original: s,
                is_negative,
                case_sensitive: false,
            },
        }
    }

    pub fn as_case_sensitive(mut self) -> Self {
        self.case_sensitive = true;
        self
    }

    pub fn is_regex(&self) -> bool {
        match &self.data {
            TagWrapperData::Raw(_) => false,
            TagWrapperData::Regex(_) => true,
        }
    }

    pub fn matches<S: AsRef<str>>(&self, haystack: S) -> bool {
        let matches = self.is_contained_within(haystack);
        if self.is_negative {
            return !matches;
        }
        matches
    }

    // You likely want matches, which considers whether the input is "negative" or not.
    // This ignores that and just returns whether the needle is found inside the haystack.
    pub fn is_contained_within<S: AsRef<str>>(&self, haystack: S) -> bool {
        let haystack = if self.case_sensitive {
            haystack.as_ref()
        } else {
            &haystack.as_ref().to_lowercase()
        };

        match &self.data {
            TagWrapperData::Raw(value) => haystack.contains(value),
            TagWrapperData::Regex(regex) => regex.is_match(haystack),
        }
    }

    pub fn replace(&self, str: String, to_string: impl Fn(&str) -> String + 'static) -> String {
        let mut output = str;
        match &self.data {
            TagWrapperData::Raw(value) => {
                let replacement = to_string(value);
                output = output.replace(value, &replacement);
            }
            TagWrapperData::Regex(regex) => {
                let highlighter = Highlighter {
                    to_string_cb: Box::new(to_string),
                };

                // TODO: Silly hack since replace_all doesn't seem to span multiple lines
                output = output.replace("\n", "abcdefg");
                output = regex.replace_all(&output, highlighter).to_string();
                output = output.replace("abcdefg", "\n");
            }
        };
        output
    }

    pub fn to_str(&self) -> &str {
        self.original.as_str()
    }

    pub fn to_string(&self) -> String {
        self.original.clone()
    }

    pub fn match_indices<S: AsRef<str>>(&self, other: S) -> Vec<(usize, usize)> {
        let other = if self.case_sensitive {
            other.as_ref()
        } else {
            &other.as_ref().to_lowercase()
        };

        match &self.data {
            TagWrapperData::Raw(value) => other
                .match_indices(value)
                .map(|(index, _)| (index, value.len()))
                .collect(),
            TagWrapperData::Regex(regex) => regex
                .find_iter(other)
                .map(|some_match| (some_match.start(), some_match.len()))
                .collect(),
        }
    }

    pub fn matches_exactly<S: AsRef<str>>(&self, other: S) -> bool {
        let other = if self.case_sensitive {
            other.as_ref()
        } else {
            &other.as_ref().to_lowercase()
        };

        match &self.data {
            TagWrapperData::Raw(value) => other == *value,
            TagWrapperData::Regex(regex) => {
                if let Some(found) = regex.find(other) {
                    return found.len() == other.len();
                }
                false
            }
        }
    }

    pub fn starts_with<S: AsRef<str>>(&self, s: S) -> bool {
        let s = if self.case_sensitive {
            s.as_ref()
        } else {
            &s.as_ref().to_lowercase()
        };

        match &self.data {
            TagWrapperData::Raw(value) => value.starts_with(s),
            TagWrapperData::Regex(regex) => {
                if let Some(found) = regex.find(s) {
                    return found.start() == 0;
                }
                false
            }
        }
    }
}

fn get_regex(s: &str) -> Option<Regex> {
    if REGEX_REGEX.is_match(s) {
        match Regex::new(s) {
            Ok(regex) => {
                return Some(regex);
            }
            Err(_e) => {
                error!("Bad regex: {s}");
            }
        }
    }
    None
}

struct Highlighter {
    to_string_cb: Box<dyn Fn(&str) -> String>,
}

impl Replacer for Highlighter {
    fn replace_append(&mut self, caps: &Captures<'_>, dst: &mut String) {
        let temp = caps.get(0).map_or("", |m| m.as_str()).to_string();
        let rv = (*self.to_string_cb)(&temp);
        dst.push_str(&rv);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn detects_regexes() {
        assert!(MaybeRegex::new("This is a regex.*").is_regex());
        assert!(MaybeRegex::new(".*This is a regex").is_regex());
        assert!(MaybeRegex::new(".This is a regex").is_regex());
        assert!(MaybeRegex::new("This is a regex [0-9]").is_regex());
    }

    #[test]
    fn detects_non_regexes() {
        assert!(!MaybeRegex::new("This is not a regex").is_regex());
        assert!(!MaybeRegex::new("This is not a regex?").is_regex());
        assert!(!MaybeRegex::new("This is not a regex [").is_regex());
        assert!(!MaybeRegex::new("This is not a regex [0-9").is_regex());
    }

    #[test]
    fn contains_works() {
        assert!(!MaybeRegex::new("z").is_contained_within("Hello"));
        assert!(!MaybeRegex::new("e$").is_contained_within("Hello"));

        assert!(MaybeRegex::new("e").is_contained_within("Hello"));
        assert!(MaybeRegex::new("o$").is_contained_within("Hello"));
    }

    #[test]
    fn negative_works() {
        assert!(MaybeRegex::new("-e").is_contained_within("Hello"));
        assert!(!MaybeRegex::new("-e").matches("Hello"));

        assert!(MaybeRegex::new("-o$").is_contained_within("Hello"));
        assert!(!MaybeRegex::new("-o$").matches("Hello"));
    }

    #[test]
    fn all_string_types_work() {
        assert!(MaybeRegex::new("e").is_contained_within("Hello"));
        assert!(MaybeRegex::new(String::from("e")).is_contained_within("Hello"));
        assert!(MaybeRegex::new(&String::from("e")).is_contained_within("Hello"));
    }
}
