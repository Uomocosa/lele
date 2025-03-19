pub fn get_substring_between<'a>(input: &'a str, start: &'a str, end: &'a str) -> Option<&'a str> {
    if let Some(start_index) = input.find(start) {
        let start = start_index + start.len();
        if let Some(end_index) = input[start..].find(end) {
            return Some(&input[start..start + end_index]);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_1() {
        todo!();
    }
}
