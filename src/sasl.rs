//! SASL authentication helpers for IRC.
//!
//! This module provides utilities for encoding SASL authentication
//! credentials using common mechanisms.
//!
//! # Supported Mechanisms
//!
//! - **PLAIN**: Simple username/password authentication (RFC 4616)
//! - **EXTERNAL**: Certificate-based authentication (client cert)
//!
//! # Reference
//! - IRCv3 SASL: <https://ircv3.net/specs/extensions/sasl-3.2>
//! - RFC 4616 (PLAIN): <https://tools.ietf.org/html/rfc4616>
//!
//! # Example
//!
//! ```
//! use slirc_proto::sasl::{SaslMechanism, encode_plain};
//!
//! // Encode PLAIN credentials
//! let encoded = encode_plain("myuser", "mypassword");
//! assert!(!encoded.is_empty());
//!
//! // Check mechanism support
//! let mech = SaslMechanism::parse("PLAIN");
//! assert_eq!(mech, SaslMechanism::Plain);
//! ```

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};

/// Maximum length of a single SASL message chunk (400 bytes).
///
/// SASL responses that exceed this length must be split into multiple
/// AUTHENTICATE commands.
pub const SASL_CHUNK_SIZE: usize = 400;

/// Supported SASL authentication mechanisms.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SaslMechanism {
    /// PLAIN mechanism (RFC 4616) - simple username/password.
    Plain,
    /// EXTERNAL mechanism - uses TLS client certificate.
    External,
    /// SCRAM-SHA-256 mechanism (RFC 7677).
    ScramSha256,
    /// Unknown or unsupported mechanism.
    Unknown(String),
}

impl SaslMechanism {
    /// Parse a mechanism name string.
    pub fn parse(name: &str) -> Self {
        match name.to_ascii_uppercase().as_str() {
            "PLAIN" => Self::Plain,
            "EXTERNAL" => Self::External,
            "SCRAM-SHA-256" => Self::ScramSha256,
            _ => Self::Unknown(name.to_owned()),
        }
    }

    /// Returns the canonical name of this mechanism.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Plain => "PLAIN",
            Self::External => "EXTERNAL",
            Self::ScramSha256 => "SCRAM-SHA-256",
            Self::Unknown(s) => s,
        }
    }

    /// Check if this mechanism is supported for encoding.
    pub fn is_supported(&self) -> bool {
        matches!(self, Self::Plain | Self::External)
    }
}

impl std::fmt::Display for SaslMechanism {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Parse a list of mechanisms from a server's `RPL_SASLMECHS` (908) response.
///
/// The mechanisms are typically comma-separated.
///
/// # Example
///
/// ```
/// use slirc_proto::sasl::{parse_mechanisms, SaslMechanism};
///
/// let mechs = parse_mechanisms("PLAIN,EXTERNAL,SCRAM-SHA-256");
/// assert!(mechs.contains(&SaslMechanism::Plain));
/// assert!(mechs.contains(&SaslMechanism::External));
/// ```
pub fn parse_mechanisms(list: &str) -> Vec<SaslMechanism> {
    list.split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(SaslMechanism::parse)
        .collect()
}

/// Choose the best supported mechanism from a list.
///
/// Preference order: EXTERNAL > SCRAM-SHA-256 > PLAIN
///
/// # Example
///
/// ```
/// use slirc_proto::sasl::{choose_mechanism, SaslMechanism};
///
/// let available = vec![
///     SaslMechanism::Plain,
///     SaslMechanism::External,
/// ];
/// assert_eq!(choose_mechanism(&available), Some(SaslMechanism::External));
/// ```
pub fn choose_mechanism(available: &[SaslMechanism]) -> Option<SaslMechanism> {
    // Prefer EXTERNAL (certificate-based) over password-based
    if available.contains(&SaslMechanism::External) {
        return Some(SaslMechanism::External);
    }
    // SCRAM-SHA-256 is more secure than PLAIN
    if available.contains(&SaslMechanism::ScramSha256) {
        return Some(SaslMechanism::ScramSha256);
    }
    // Fall back to PLAIN
    if available.contains(&SaslMechanism::Plain) {
        return Some(SaslMechanism::Plain);
    }
    None
}

/// Encode credentials for the PLAIN mechanism.
///
/// The PLAIN mechanism encodes: `authzid NUL authcid NUL password`
///
/// For IRC SASL, `authzid` is typically empty and `authcid` is the username.
///
/// # Arguments
///
/// * `username` - The authentication identity (authcid)
/// * `password` - The password
///
/// # Returns
///
/// Base64-encoded PLAIN authentication string.
///
/// # Example
///
/// ```
/// use slirc_proto::sasl::encode_plain;
///
/// let encoded = encode_plain("testuser", "testpass");
/// // Decodes to: "\0testuser\0testpass"
/// assert!(!encoded.is_empty());
/// ```
pub fn encode_plain(username: &str, password: &str) -> String {
    // Format: authzid NUL authcid NUL password
    // For IRC, authzid is typically empty
    let payload = format!("\0{}\0{}", username, password);
    BASE64.encode(payload.as_bytes())
}

/// Encode credentials for the PLAIN mechanism with an explicit authzid.
///
/// Use this when you need to authenticate as one user but authorize as another.
///
/// # Arguments
///
/// * `authzid` - The authorization identity (who to act as)
/// * `authcid` - The authentication identity (who is authenticating)
/// * `password` - The password
pub fn encode_plain_with_authzid(authzid: &str, authcid: &str, password: &str) -> String {
    let payload = format!("{}\0{}\0{}", authzid, authcid, password);
    BASE64.encode(payload.as_bytes())
}

/// Encode an EXTERNAL mechanism response.
///
/// For EXTERNAL, the response is typically empty ("+") or contains
/// the authorization identity if different from the certificate CN.
///
/// # Arguments
///
/// * `authzid` - Optional authorization identity. Pass `None` for default.
pub fn encode_external(authzid: Option<&str>) -> String {
    match authzid {
        Some(id) if !id.is_empty() => BASE64.encode(id.as_bytes()),
        _ => "+".to_owned(), // Empty response
    }
}

/// Split an encoded SASL response into chunks for transmission.
///
/// IRC SASL requires responses longer than 400 bytes to be split
/// across multiple AUTHENTICATE commands.
///
/// # Example
///
/// ```
/// use slirc_proto::sasl::chunk_response;
///
/// let response = "a]".repeat(250); // Long response
/// let chunks: Vec<_> = chunk_response(&response).collect();
/// assert!(chunks.len() > 1);
/// for chunk in &chunks[..chunks.len()-1] {
///     assert_eq!(chunk.len(), 400);
/// }
/// ```
pub fn chunk_response(encoded: &str) -> impl Iterator<Item = &str> {
    encoded.as_bytes().chunks(SASL_CHUNK_SIZE).map(|chunk| {
        // Safe because base64 is always ASCII
        std::str::from_utf8(chunk).unwrap()
    })
}

/// Check if a SASL response needs chunking.
#[inline]
pub fn needs_chunking(encoded: &str) -> bool {
    encoded.len() > SASL_CHUNK_SIZE
}

/// Decode a base64-encoded SASL challenge or response.
///
/// # Returns
///
/// The decoded bytes, or an error if decoding fails.
pub fn decode_base64(encoded: &str) -> Result<Vec<u8>, base64::DecodeError> {
    if encoded == "+" {
        return Ok(Vec::new());
    }
    BASE64.decode(encoded)
}

/// SASL authentication state machine.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SaslState {
    /// Initial state, not yet started.
    Initial,
    /// Sent AUTHENTICATE with mechanism, waiting for challenge.
    MechanismSent(SaslMechanism),
    /// Received challenge, need to send credentials.
    ChallengeReceived,
    /// Sent credentials, waiting for result.
    CredentialsSent,
    /// Authentication succeeded.
    Success,
    /// Authentication failed.
    Failed(String),
    /// Authentication aborted.
    Aborted,
}

impl SaslState {
    /// Check if authentication is complete (success or failure).
    pub fn is_complete(&self) -> bool {
        matches!(self, Self::Success | Self::Failed(_) | Self::Aborted)
    }

    /// Check if authentication succeeded.
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_plain() {
        let encoded = encode_plain("testuser", "testpass");
        let decoded = BASE64.decode(&encoded).unwrap();
        assert_eq!(decoded, b"\0testuser\0testpass");
    }

    #[test]
    fn test_encode_plain_with_authzid() {
        let encoded = encode_plain_with_authzid("admin", "testuser", "testpass");
        let decoded = BASE64.decode(&encoded).unwrap();
        assert_eq!(decoded, b"admin\0testuser\0testpass");
    }

    #[test]
    fn test_encode_external_empty() {
        let encoded = encode_external(None);
        assert_eq!(encoded, "+");
    }

    #[test]
    fn test_encode_external_with_authzid() {
        let encoded = encode_external(Some("myuser"));
        let decoded = BASE64.decode(&encoded).unwrap();
        assert_eq!(decoded, b"myuser");
    }

    #[test]
    fn test_parse_mechanisms() {
        let mechs = parse_mechanisms("PLAIN,EXTERNAL,SCRAM-SHA-256");
        assert_eq!(mechs.len(), 3);
        assert!(mechs.contains(&SaslMechanism::Plain));
        assert!(mechs.contains(&SaslMechanism::External));
        assert!(mechs.contains(&SaslMechanism::ScramSha256));
    }

    #[test]
    fn test_choose_mechanism_prefers_external() {
        let available = vec![SaslMechanism::Plain, SaslMechanism::External];
        assert_eq!(choose_mechanism(&available), Some(SaslMechanism::External));
    }

    #[test]
    fn test_choose_mechanism_prefers_scram_over_plain() {
        let available = vec![SaslMechanism::Plain, SaslMechanism::ScramSha256];
        assert_eq!(
            choose_mechanism(&available),
            Some(SaslMechanism::ScramSha256)
        );
    }

    #[test]
    fn test_choose_mechanism_plain_fallback() {
        let available = vec![SaslMechanism::Plain];
        assert_eq!(choose_mechanism(&available), Some(SaslMechanism::Plain));
    }

    #[test]
    fn test_choose_mechanism_none() {
        let available = vec![SaslMechanism::Unknown("FOO".to_owned())];
        assert_eq!(choose_mechanism(&available), None);
    }

    #[test]
    fn test_chunk_response_short() {
        let short = "abc123";
        let chunks: Vec<_> = chunk_response(short).collect();
        assert_eq!(chunks, vec!["abc123"]);
    }

    #[test]
    fn test_chunk_response_long() {
        let long = "a".repeat(500);
        let chunks: Vec<_> = chunk_response(&long).collect();
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].len(), 400);
        assert_eq!(chunks[1].len(), 100);
    }

    #[test]
    fn test_needs_chunking() {
        assert!(!needs_chunking("short"));
        assert!(needs_chunking(&"a".repeat(500)));
    }

    #[test]
    fn test_decode_base64_empty() {
        let decoded = decode_base64("+").unwrap();
        assert!(decoded.is_empty());
    }

    #[test]
    fn test_decode_base64_valid() {
        let encoded = BASE64.encode(b"hello");
        let decoded = decode_base64(&encoded).unwrap();
        assert_eq!(decoded, b"hello");
    }

    #[test]
    fn test_mechanism_parse() {
        assert_eq!(SaslMechanism::parse("PLAIN"), SaslMechanism::Plain);
        assert_eq!(SaslMechanism::parse("plain"), SaslMechanism::Plain);
        assert_eq!(SaslMechanism::parse("EXTERNAL"), SaslMechanism::External);
        assert_eq!(
            SaslMechanism::parse("SCRAM-SHA-256"),
            SaslMechanism::ScramSha256
        );
        assert_eq!(
            SaslMechanism::parse("UNKNOWN"),
            SaslMechanism::Unknown("UNKNOWN".to_owned())
        );
    }

    #[test]
    fn test_mechanism_as_str() {
        assert_eq!(SaslMechanism::Plain.as_str(), "PLAIN");
        assert_eq!(SaslMechanism::External.as_str(), "EXTERNAL");
        assert_eq!(SaslMechanism::ScramSha256.as_str(), "SCRAM-SHA-256");
    }

    #[test]
    fn test_mechanism_is_supported() {
        assert!(SaslMechanism::Plain.is_supported());
        assert!(SaslMechanism::External.is_supported());
        assert!(!SaslMechanism::ScramSha256.is_supported());
        assert!(!SaslMechanism::Unknown("FOO".to_owned()).is_supported());
    }

    #[test]
    fn test_sasl_state() {
        assert!(!SaslState::Initial.is_complete());
        assert!(!SaslState::MechanismSent(SaslMechanism::Plain).is_complete());
        assert!(SaslState::Success.is_complete());
        assert!(SaslState::Success.is_success());
        assert!(SaslState::Failed("error".to_owned()).is_complete());
        assert!(!SaslState::Failed("error".to_owned()).is_success());
        assert!(SaslState::Aborted.is_complete());
    }
}
