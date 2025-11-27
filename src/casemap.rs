
pub fn irc_to_lower(s: &str) -> String {
    s.chars()
        .map(|c| match c {

            '[' => '{',
            ']' => '}',
            '\\' => '|',
            '~' => '^',
            
            'A'..='Z' => c.to_ascii_lowercase(),
            
            _ => c,
        })
        .collect()
}

pub fn irc_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    a.chars().zip(b.chars()).all(|(ca, cb)| {
        let ca_lower = match ca {
            '[' => '{',
            ']' => '}',
            '\\' => '|',
            '~' => '^',
            'A'..='Z' => ca.to_ascii_lowercase(),
            _ => ca,
        };
        let cb_lower = match cb {
            '[' => '{',
            ']' => '}',
            '\\' => '|',
            '~' => '^',
            'A'..='Z' => cb.to_ascii_lowercase(),
            _ => cb,
        };
        ca_lower == cb_lower
    })
}

