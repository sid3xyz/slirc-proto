//! SCRAM-SHA-256 SASL mechanism (RFC 7677).
//!
//! Challenge-response authentication mechanism with partial support.
//!
//! # SCRAM Support
//!
//! SCRAM-SHA-256 is recognized and preferred, but full client-side
//! implementation requires cryptographic dependencies (sha2, hmac, pbkdf2).
//! The [`ScramClient`] struct provides the state machine; actual payload
//! generation will be added in a future release with an optional `scram`
//! feature flag.
//!
//! # SCRAM Protocol Flow
//!
//! 1. Client sends `client-first-message`: `n,,n=user,r=nonce`
//! 2. Server sends `server-first-message`: `r=nonce+server,s=salt,i=iterations`
//! 3. Client sends `client-final-message`: `c=biws,r=nonce+server,p=proof`
//! 4. Server sends `server-final-message`: `v=verifier`
//!
//! # Reference
//! - RFC 7677: <https://tools.ietf.org/html/rfc7677>

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};

use super::decode_base64;

/// SCRAM-SHA-256 client state machine.
///
/// This provides the state machine for SCRAM authentication. Full payload
/// generation requires enabling the `scram` feature (not yet implemented).
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
