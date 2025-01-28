use regex;

pub fn extract_number(file_path: &str) -> Option<u32> {
    // Use a regular expression to find the numeric part
    let re = regex::Regex::new(r"\d+").unwrap();

    // Search for the first match
    if let Some(captures) = re.captures(file_path) {
        // Parse the matched string into a number
        captures.get(0).and_then(|m| m.as_str().parse::<u32>().ok())
    } else {
        None
    }
}
