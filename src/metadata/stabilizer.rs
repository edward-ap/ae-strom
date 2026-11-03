/// Keeps the last stable title and filters out repeats and empty values.
///
/// Used to deduplicate metadata updates and avoid redundant UI refreshes
/// when the same title is received multiple times.
#[derive(Clone)]
pub struct TitleStabilizer {
    current_title: String,
}

impl TitleStabilizer {
    /// Create a new title stabilizer with an empty initial state.
    pub fn new() -> Self {
        Self {
            current_title: String::new(),
        }
    }

    /// Process a new raw title. Returns Some(clean) if non-empty and different.
    ///
    /// Cleans the title by trimming whitespace and decoding common HTML entities.
    /// Returns None if the title is empty, unchanged, or consists only of "-".
    pub fn on_new_title(&mut self, raw: &str) -> Option<String> {
        let clean = Self::clean(raw);
        if clean.is_empty() {
            return None;
        }
        if clean == self.current_title {
            return None;
        }
        self.current_title = clean.clone();
        Some(clean)
    }

    /// Clean and normalize a raw title string.
    ///
    /// Trims whitespace, replaces "-" with empty string, and decodes
    /// common HTML entities (&amp;, &apos;, &quot;).
    fn clean(raw: &str) -> String {
        let mut s = raw.trim().to_string();
        if s == "-" {
            return String::new();
        }
        s = s
            .replace("&amp;", "&")
            .replace("&apos;", "'")
            .replace("&quot;", "\"");
        s
    }
}
