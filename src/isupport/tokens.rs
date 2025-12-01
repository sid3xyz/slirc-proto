//! ISUPPORT builder for server responses.

/// Builder for constructing ISUPPORT token strings.
///
/// Useful for IRC servers to generate `RPL_ISUPPORT` responses.
#[derive(Debug, Clone, Default)]
pub struct IsupportBuilder {
    tokens: Vec<String>,
}

impl IsupportBuilder {
    /// Create a new empty builder.
    pub fn new() -> Self {
        Self { tokens: Vec::new() }
    }

    /// Set the `NETWORK` token.
    pub fn network(mut self, name: &str) -> Self {
        self.tokens.push(format!("NETWORK={}", name));
        self
    }

    /// Set the `CHANTYPES` token.
    pub fn chantypes(mut self, types: &str) -> Self {
        self.tokens.push(format!("CHANTYPES={}", types));
        self
    }

    /// Set the `CHANMODES` token.
    pub fn chanmodes(mut self, modes: &str) -> Self {
        self.tokens.push(format!("CHANMODES={}", modes));
        self
    }

    /// Set the `PREFIX` token with symbols and mode letters.
    pub fn prefix(mut self, symbols: &str, letters: &str) -> Self {
        self.tokens.push(format!("PREFIX=({}){}", letters, symbols));
        self
    }

    /// Set the `CASEMAPPING` token.
    pub fn casemapping(mut self, mapping: &str) -> Self {
        self.tokens.push(format!("CASEMAPPING={}", mapping));
        self
    }

    /// Set the `MAXCHANNELS` token.
    pub fn max_channels(mut self, count: u32) -> Self {
        self.tokens.push(format!("MAXCHANNELS={}", count));
        self
    }

    /// Set the `NICKLEN` token.
    pub fn max_nick_length(mut self, len: u32) -> Self {
        self.tokens.push(format!("NICKLEN={}", len));
        self
    }

    /// Set the `TOPICLEN` token.
    pub fn max_topic_length(mut self, len: u32) -> Self {
        self.tokens.push(format!("TOPICLEN={}", len));
        self
    }

    /// Set the `MODES` token (max modes per command).
    pub fn modes_count(mut self, count: u32) -> Self {
        self.tokens.push(format!("MODES={}", count));
        self
    }

    /// Set the `STATUSMSG` token.
    pub fn status_msg(mut self, symbols: &str) -> Self {
        self.tokens.push(format!("STATUSMSG={}", symbols));
        self
    }

    /// Set the `EXCEPTS` token.
    pub fn excepts(mut self, mode_char: Option<char>) -> Self {
        if let Some(c) = mode_char {
            self.tokens.push(format!("EXCEPTS={}", c));
        } else {
            self.tokens.push("EXCEPTS".to_string());
        }
        self
    }

    /// Set the `INVEX` token.
    pub fn invex(mut self, mode_char: Option<char>) -> Self {
        if let Some(c) = mode_char {
            self.tokens.push(format!("INVEX={}", c));
        } else {
            self.tokens.push("INVEX".to_string());
        }
        self
    }

    /// Add a custom token.
    pub fn custom(mut self, key: &str, value: Option<&str>) -> Self {
        if let Some(v) = value {
            self.tokens.push(format!("{}={}", key, v));
        } else {
            self.tokens.push(key.to_string());
        }
        self
    }

    /// Build the tokens into a single space-separated string.
    pub fn build(self) -> String {
        self.tokens.join(" ")
    }

    /// Build the tokens into multiple lines, each with at most `max_per_line` tokens.
    pub fn build_lines(self, max_per_line: usize) -> Vec<String> {
        let mut lines = Vec::new();
        let mut current = Vec::new();

        for token in self.tokens {
            current.push(token);
            if current.len() >= max_per_line {
                lines.push(current.join(" "));
                current.clear();
            }
        }

        if !current.is_empty() {
            lines.push(current.join(" "));
        }

        lines
    }
}
