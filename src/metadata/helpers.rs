/// Normalize stream titles to a simple CamelCase / TitleCase form:
/// - lowercases everything except first letter of each word
/// - words start after whitespace or common separators (- _ / \)
/// - keeps punctuation/apostrophes intact (e.g., "n'roses" -> "N'roses").
pub fn normalize_stream_title_case(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut new_word = true;
    for ch in input.chars() {
        if ch.is_whitespace() || "-_/\\(".contains(ch) {
            new_word = true;
            out.push(ch);
            continue;
        }
        if ch.is_alphabetic() {
            if new_word {
                out.extend(ch.to_uppercase());
            } else {
                out.extend(ch.to_lowercase());
            }
            new_word = false;
        } else {
            out.push(ch);
            // Apostrophes and similar keep the word going; others start a new word.
            if ch != '\'' {
                new_word = true;
            }
        }
    }
    out.trim().to_string()
}
