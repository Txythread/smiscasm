pub fn remove_comments_in_line(text: String) -> String {
    let mut output = String::new();

    for char in text.chars() {
        if char == '#' { break; }
        output.push(char);
    }

    output
}