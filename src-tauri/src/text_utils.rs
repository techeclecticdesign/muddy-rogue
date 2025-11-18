pub fn wrap_lines(text: &str, line_length: usize) -> Vec<String> {
    let mut lines = Vec::new();

    // Handle explicit newlines in the source text first
    for paragraph in text.split('\n') {
        if paragraph.trim().is_empty() {
            // Preserve empty lines
            lines.push(String::new());
            continue;
        }

        let mut current_line = String::new();
        let words: Vec<&str> = paragraph.split_whitespace().collect();

        for word in words {
            if current_line.is_empty() {
                current_line.push_str(word);
            } else {
                // Check if adding the next word exceeds the limit
                if current_line.len() + 1 + word.len() > line_length {
                    lines.push(current_line);
                    current_line = String::from(word);
                } else {
                    current_line.push(' ');
                    current_line.push_str(word);
                }
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }
    }

    lines
}

pub fn format_list(items: &[String]) -> String {
    match items.len() {
        0 => String::new(),
        1 => items[0].clone(),
        2 => format!("{} and {}", items[0], items[1]),
        _ => {
            let (last, rest) = items.split_last().unwrap();
            format!("{}, and {}", rest.join(", "), last)
        }
    }
}
