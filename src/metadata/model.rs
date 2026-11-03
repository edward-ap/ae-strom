/// Hint about which metadata strategy to try first.
#[derive(Debug, Clone)]
pub struct StrategyHint {
    /// "ICY" or "JSON" or empty
    pub r#type: String,
    /// URL where metadata should be fetched.
    pub url: String,
}

/// Normalized metadata payload that UI code consumes.
#[derive(Debug, Clone)]
pub struct Info {
    /// Track title (Now Playing).
    pub title: String,
    /// Station name, typically from the `icy-name` response header.
    pub station_name: Option<String>,
}
