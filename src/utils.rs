pub fn remove_first_n_chars(s: &str, n: usize) -> String {
    return s.chars().skip(n).collect();
}

pub fn remove_last_n_chars(s: &str, n: usize) -> String {
    let mut chars: Vec<char> = s.chars().collect();
    for _ in 0..n {
        chars.pop();
    }
    return chars.iter().collect();
}
