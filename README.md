# maybe-regex

Regex is amazing, but it's also slower than a plain-text search. This is a simple utility that wraps some generic needle, attempts to detect if it looks regex-y, and then provides some useful functions for using it.

## Usage

```rust
// Regexes work as you'd expect
assert_eq!(MaybeRegex::new("e$").matches("Hello"), false);

// As do plain strings
assert_eq!(MaybeRegex::new("e").matches("Hello"), true);

// Plain string search is case insensitive by default
assert_eq!(MaybeRegex::new("h").matches("Hello"), true);

// ...though that can be disabled
assert_eq!(MaybeRegex::new("h").as_case_sensitive().matches("Hello"), false);

// Strings that start or end with a '-' are understood to be "negative".
// So haystacks that contain the string won't match and vice-versa.
assert_eq!(MaybeRegex::new("-e").matches("Hello"), false);

// You can ignore "negative" behavior by using the 'is_contained_within' method.
assert_eq!(MaybeRegex::new("-e").is_contained_within("Hello"), true);
```

## Performance

It's about what you'd expect, roughly as fast as a regex for regexes or plain strings for plain strings.

The most expensive feature, case insensitivity by default, can be disabled if you'd like:

```rust
let needle = MaybeRegex::new("o$").as_case_sensitive();
```
