//! SASL authentication helpers for IRC.
//!
//! This module provides utilities for encoding SASL authentication
//! credentials using common mechanisms.
//!
//! # Supported Mechanisms
//!
//! - **PLAIN**: Simple username/password authentication (RFC 4616)
//! - **EXTERNAL**: Certificate-based authentication (client cert)
//! - **SCRAM-SHA-256**: Challenge-response authentication (RFC 7677) - *partial support*
//!
//! # SCRAM-SHA-256 Support
//!
//! SCRAM-SHA-256 is recognized and preferred by [`choose_mechanism`], but full
//! client-side implementation requires cryptographic dependencies (sha2, hmac, pbkdf2).
//! The [`ScramClient`] struct provides the state machine; actual payload generation
//! will be added in a future release with an optional `scram` feature flag.
//!
//! # Reference
//! - IRCv3 SASL: <https://ircv3.net/specs/extensions/sasl-3.2>
//! - RFC 4616 (PLAIN): <https://tools.ietf.org/html/rfc4616>
//! - RFC 7677 (SCRAM-SHA-256): <https://tools.ietf.org/html/rfc7677>
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

/// SCRAM-SHA-256 client state machine.
///
/// This provides the state machine for SCRAM authentication. Full payload
/// generation requires enabling the `scram` feature (not yet implemented).
///
/// # SCRAM Protocol Flow
///
/// 1. Client sends `client-first-message`: `n,,n=user,r=nonce`
/// 2. Server sends `server-first-message`: `r=nonce+server,s=salt,i=iterations`
/// 3. Client sends `client-final-message`: `c=biws,r=nonce+server,p=proof`
/// 4. Server sends `server-final-message`: `v=verifier`
///
/// # Example
///
/// ```
/// use slirc_proto::sasl::ScramClient;
///
/// let mut client = ScramClient::new("username", "password");
/// let first_message = client.client_first_message();
/// // Send first_message to server via AUTHENTICATE
/// ```
#[derive(Clone, Debug)]
pub struct ScramClient {
    username: String,
    /// Password for SCRAM authentication.
    /// Not yet used - full SCRAM requires crypto dependencies (sha2, hmac, pbkdf2).
    #[allow(dead_code)]
    password: String,
    client_nonce: String,
    state: ScramState,
}

/// Internal state of SCRAM authentication.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScramState {
    /// Initial state.
    Initial,
    /// Sent client-first, awaiting server-first.
    ClientFirstSent,
    /// Received server-first, ready to send client-final.
    ServerFirstReceived {
        /// Combined nonce (client + server).
        nonce: String,
        /// Salt from server (base64 decoded).
        salt: Vec<u8>,
        /// Iteration count.
        iterations: u32,
    },
    /// Sent client-final, awaiting server-final.
    ClientFinalSent,
    /// Authentication complete.
    Complete,
    /// Authentication failed.
    Failed(String),
}

impl ScramClient {
    /// Create a new SCRAM client with the given credentials.
    #[must_use]
    pub fn new(username: &str, password: &str) -> Self {
        // Generate a random nonce (in production, use a CSPRNG)
        let nonce = generate_nonce();

        Self {
            username: username.to_string(),
            password: password.to_string(),
            client_nonce: nonce,
            state: ScramState::Initial,
        }
    }

    /// Get the current SCRAM state.
    #[must_use]
    pub fn state(&self) -> &ScramState {
        &self.state
    }

    /// Generate the client-first-message.
    ///
    /// This is the first message sent to the server after AUTHENTICATE SCRAM-SHA-256.
    /// Returns a base64-encoded message ready for transmission.
    #[must_use]
    pub fn client_first_message(&mut self) -> String {
        self.state = ScramState::ClientFirstSent;

        // gs2-header: n,, (no channel binding, no authzid)
        // client-first-message-bare: n=username,r=nonce
        let bare = format!("n={},r={}", saslprep(&self.username), self.client_nonce);
        let full = format!("n,,{}", bare);

        BASE64.encode(full.as_bytes())
    }

    /// Process the server-first-message and generate client-final-message.
    ///
    /// # Arguments
    ///
    /// * `server_first` - The base64-encoded server-first-message.
    ///
    /// # Returns
    ///
    /// The base64-encoded client-final-message, or an error.
    ///
    /// # Note
    ///
    /// Full implementation requires cryptographic operations (PBKDF2, HMAC-SHA256).
    /// This is a placeholder that returns an error indicating the feature is not available.
    pub fn process_server_first(&mut self, server_first: &str) -> Result<String, ScramError> {
        let decoded = decode_base64(server_first).map_err(|_| ScramError::InvalidEncoding)?;
        let message = String::from_utf8(decoded).map_err(|_| ScramError::InvalidEncoding)?;

        // Parse server-first-message: r=nonce,s=salt,i=iterations
        let mut nonce = None;
        let mut salt = None;
        let mut iterations = None;

        for part in message.split(',') {
            if let Some(value) = part.strip_prefix("r=") {
                nonce = Some(value.to_string());
            } else if let Some(value) = part.strip_prefix("s=") {
                salt = Some(decode_base64(value).map_err(|_| ScramError::InvalidEncoding)?);
            } else if let Some(value) = part.strip_prefix("i=") {
                iterations = Some(value.parse().map_err(|_| ScramError::InvalidIterations)?);
            }
        }

        let nonce = nonce.ok_or(ScramError::MissingNonce)?;
        let salt = salt.ok_or(ScramError::MissingSalt)?;
        let iterations = iterations.ok_or(ScramError::MissingIterations)?;

        // Verify that server nonce starts with our client nonce
        if !nonce.starts_with(&self.client_nonce) {
            return Err(ScramError::NonceMismatch);
        }

        self.state = ScramState::ServerFirstReceived {
            nonce: nonce.clone(),
            salt,
            iterations,
        };

        // Note: Full implementation would compute:
        // - SaltedPassword = Hi(password, salt, iterations)  // PBKDF2
        // - ClientKey = HMAC(SaltedPassword, "Client Key")
        // - StoredKey = SHA256(ClientKey)
        // - ClientSignature = HMAC(StoredKey, AuthMessage)
        // - ClientProof = ClientKey XOR ClientSignature
        //
        // For now, return an error indicating crypto is not available
        Err(ScramError::CryptoNotAvailable)
    }

    /// Verify the server-final-message.
    ///
    /// # Note
    ///
    /// Requires cryptographic verification. Placeholder implementation.
    pub fn verify_server_final(&mut self, _server_final: &str) -> Result<(), ScramError> {
        Err(ScramError::CryptoNotAvailable)
    }
}

/// Errors that can occur during SCRAM authentication.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScramError {
    /// Base64 decoding failed.
    InvalidEncoding,
    /// Server nonce doesn't match client nonce prefix.
    NonceMismatch,
    /// Missing nonce in server message.
    MissingNonce,
    /// Missing salt in server message.
    MissingSalt,
    /// Missing iteration count in server message.
    MissingIterations,
    /// Invalid iteration count.
    InvalidIterations,
    /// Server verification failed.
    ServerVerificationFailed,
    /// Cryptographic operations not available (requires `scram` feature).
    CryptoNotAvailable,
}

impl std::fmt::Display for ScramError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidEncoding => write!(f, "invalid base64 encoding"),
            Self::NonceMismatch => write!(f, "server nonce doesn't match client nonce"),
            Self::MissingNonce => write!(f, "missing nonce in server message"),
            Self::MissingSalt => write!(f, "missing salt in server message"),
            Self::MissingIterations => write!(f, "missing iteration count"),
            Self::InvalidIterations => write!(f, "invalid iteration count"),
            Self::ServerVerificationFailed => write!(f, "server verification failed"),
            Self::CryptoNotAvailable => {
                write!(f, "SCRAM crypto not available (requires scram feature)")
            }
        }
    }
}

impl std::error::Error for ScramError {}

/// Generate a random nonce for SCRAM.
///
/// Uses a simple timestamp-based nonce. In production, use a CSPRNG.
fn generate_nonce() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    // Simple nonce based on time - not cryptographically secure
    // A real implementation should use getrandom or similar
    format!("{}_{}", now.as_nanos(), std::process::id())
}

/// Perform SASLprep normalization on a string.
///
/// This is a simplified version that handles common cases.
/// A full implementation would follow RFC 4013.
fn saslprep(s: &str) -> String {
    // For now, just pass through. A real implementation would:
    // 1. Map certain characters
    // 2. Normalize to NFKC
    // 3. Check for prohibited characters
    s.to_string()
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
