use crate::utils::{remove_first_n_chars, remove_last_n_chars};
use lazy_static::lazy_static;
use log::error;
use regex::{Captures, Regex, Replacer};
use std::cmp::Ordering;

mod utils;

lazy_static! {
    static ref REGEX_REGEX: Regex = Regex::new(r"[\\b\$\^\[\]\+\*\.]").unwrap();
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct TagWrapper {
    data: TagWrapperData,
    original: String,
    pub is_negative: bool,
}

#[derive(Debug, Clone)]
pub enum TagWrapperData {
    Raw(String),
    Regex(Regex),
}

impl PartialOrd for TagWrapperData {
    fn partial_cmp(&self, _other: &Self) -> Option<std::cmp::Ordering> {
        Some(Ordering::Equal)
    }
}

impl PartialEq for TagWrapperData {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl TagWrapper {
    pub fn from<S: Into<String>>(s: S) -> Self {
        let mut s = s.into();
        let is_negative = if s.starts_with("-") {
            s = remove_first_n_chars(&s, 1);
            true
        } else if s.ends_with("-") {
            s = remove_last_n_chars(&s, 1);
            true
        } else {
            false
        };

        match get_regex(&s) {
            Some(regex) => Self {
                data: TagWrapperData::Regex(regex),
                original: s.into(),
                is_negative,
            },
            None => Self {
                data: TagWrapperData::Raw(s.clone()),
                original: s.into(),
                is_negative,
            },
        }
    }

    pub fn matches<'a, S: Into<&'a str>>(&self, haystack: S) -> bool {
        let matches = self.is_contained_within(haystack);
        if self.is_negative {
            return !matches;
        }
        return matches;
    }

    pub fn is_contained_within<'a, S: Into<&'a str>>(&self, haystack: S) -> bool {
        // TODO: should case insensitivity be an option?
        let haystack = haystack.into().to_lowercase();
        match &self.data {
            TagWrapperData::Raw(value) => haystack.contains(value),
            TagWrapperData::Regex(regex) => regex.is_match(&haystack),
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
        return output;
    }

    pub fn to_str(&self) -> &str {
        self.original.as_str()
    }

    pub fn to_string(&self) -> String {
        self.original.clone()
    }

    pub fn match_indices(&self, other: &str) -> Vec<(usize, usize)> {
        let mut rv = vec![];
        let other = &other.to_lowercase();
        match &self.data {
            TagWrapperData::Raw(value) => {
                for (index, _) in other.match_indices(value) {
                    rv.push((index, value.len()));
                }
            }
            TagWrapperData::Regex(regex) => {
                for some_match in regex.find_iter(other) {
                    rv.push((some_match.start(), some_match.len()));
                }
            }
        };

        return rv;
    }

    pub fn matches_exactly<S: Into<String>>(&self, other: S) -> bool {
        let other = other.into().to_lowercase();
        match &self.data {
            TagWrapperData::Raw(value) => other == *value,
            TagWrapperData::Regex(regex) => {
                if let Some(found) = regex.find(&other) {
                    return found.len() == other.len();
                }
                return false;
            }
        }
    }

    pub fn starts_with<'a, S: Into<&'a str>>(&self, s: S) -> bool {
        let s = s.into().to_lowercase();
        match &self.data {
            TagWrapperData::Raw(value) => value.starts_with(&s),
            TagWrapperData::Regex(regex) => {
                if let Some(found) = regex.find(&s) {
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
    return None;
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
