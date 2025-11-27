use std::fmt;
#[cfg(feature = "tokio")]
use tokio_tungstenite::tungstenite::handshake::server::{ErrorResponse, Request, Response};
#[cfg(feature = "tokio")]
use tokio_tungstenite::tungstenite::http::StatusCode;

#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    pub allowed_origins: Vec<String>,

    pub require_origin: bool,

    pub subprotocol: Option<String>,

    pub enable_cors: bool,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            allowed_origins: Vec::new(),
            require_origin: false,
            subprotocol: Some("irc".to_string()),
            enable_cors: true,
        }
    }
}

impl WebSocketConfig {
    pub fn production() -> Self {
        Self {
            allowed_origins: Vec::new(),
            require_origin: true,
            subprotocol: Some("irc".to_string()),
            enable_cors: true,
        }
    }

    pub fn development() -> Self {
        Self {
            allowed_origins: vec![
                "http://localhost:3000".to_string(),
                "http://localhost:8080".to_string(),
                "http://127.0.0.1:3000".to_string(),
                "http://127.0.0.1:8080".to_string(),
            ],
            require_origin: false,
            subprotocol: Some("irc".to_string()),
            enable_cors: true,
        }
    }
}

#[derive(Debug)]
pub enum HandshakeResult {
    Accept {
        subprotocol: Option<String>,
        origin: Option<String>,
    },
    Reject {
        status: u16,
        reason: String,
    },
}

impl fmt::Display for HandshakeResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HandshakeResult::Accept {
                subprotocol,
                origin,
            } => {
                write!(f, "Accept")?;
                if let Some(proto) = subprotocol {
                    write!(f, " (protocol: {})", proto)?;
                }
                if let Some(orig) = origin {
                    write!(f, " (origin: {})", orig)?;
                }
                Ok(())
            }
            HandshakeResult::Reject { status, reason } => {
                write!(f, "Reject {} - {}", status, reason)
            }
        }
    }
}

#[cfg(feature = "tokio")]
pub fn validate_handshake(req: &Request, config: &WebSocketConfig) -> HandshakeResult {
    let origin = req
        .headers()
        .get("Origin")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    if config.require_origin && origin.is_none() {
        return HandshakeResult::Reject {
            status: 403,
            reason: "Origin header required".to_string(),
        };
    }

    if !config.allowed_origins.is_empty() {
        if let Some(ref origin_value) = origin {
            if !config
                .allowed_origins
                .iter()
                .any(|allowed| allowed == origin_value)
            {
                return HandshakeResult::Reject {
                    status: 403,
                    reason: format!("Origin '{}' not allowed", origin_value),
                };
            }
        }
    }

    let selected_protocol = if let Some(ref advertised_proto) = config.subprotocol {
        let requested_protocols = req
            .headers()
            .get_all("Sec-WebSocket-Protocol")
            .iter()
            .filter_map(|v| v.to_str().ok());

        let mut matched = None;
        for proto in requested_protocols {
            for p in proto.split(',').map(|s| s.trim()) {
                if p == advertised_proto {
                    matched = Some(advertised_proto.clone());
                    break;
                }
            }
            if matched.is_some() {
                break;
            }
        }
        matched
    } else {
        None
    };

    HandshakeResult::Accept {
        subprotocol: selected_protocol,
        origin,
    }
}

#[cfg(feature = "tokio")]
#[allow(clippy::result_large_err)]
pub fn build_handshake_response(
    result: &HandshakeResult,
    config: &WebSocketConfig,
) -> Result<Response, ErrorResponse> {
    match result {
        HandshakeResult::Accept {
            subprotocol,
            origin,
        } => {
            let mut builder = Response::builder().status(StatusCode::SWITCHING_PROTOCOLS);

            if config.enable_cors {
                if let Some(ref origin_value) = origin {
                    builder = builder
                        .header("Access-Control-Allow-Origin", origin_value.as_str())
                        .header("Access-Control-Allow-Credentials", "true")
                        .header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
                        .header(
                            "Access-Control-Allow-Headers",
                            "Content-Type, Authorization",
                        );
                }
            }

            if let Some(ref proto) = subprotocol {
                builder = builder.header("Sec-WebSocket-Protocol", proto.as_str());
            }

            builder
                .body(())
                .map_err(|e| ErrorResponse::new(Some(format!("Failed to build response: {}", e))))
        }
        HandshakeResult::Reject { status, reason } => Err(ErrorResponse::new(Some(format!(
            "HTTP {}: {}",
            status, reason
        )))),
    }
}

#[cfg(test)]
#[cfg(feature = "tokio")]
mod tests {
    use super::*;
    use tokio_tungstenite::tungstenite::http::Request as HttpRequest;

    fn mock_request(origin: Option<&str>, protocols: Option<&str>) -> Request {
        let mut builder = HttpRequest::builder()
            .uri("/")
            .method("GET")
            .header("Host", "localhost:6668")
            .header("Upgrade", "websocket")
            .header("Connection", "Upgrade")
            .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
            .header("Sec-WebSocket-Version", "13");

        if let Some(o) = origin {
            builder = builder.header("Origin", o);
        }

        if let Some(p) = protocols {
            builder = builder.header("Sec-WebSocket-Protocol", p);
        }

        builder.body(()).unwrap()
    }

    #[test]
    fn test_validate_accept_any_origin() {
        let config = WebSocketConfig {
            allowed_origins: Vec::new(),
            require_origin: false,
            subprotocol: Some("irc".to_string()),
            enable_cors: true,
        };

        let req = mock_request(Some("https://example.com"), Some("irc"));
        let result = validate_handshake(&req, &config);

        match result {
            HandshakeResult::Accept {
                subprotocol,
                origin,
            } => {
                assert_eq!(subprotocol, Some("irc".to_string()));
                assert_eq!(origin, Some("https://example.com".to_string()));
            }
            _ => panic!("Expected Accept"),
        }
    }

    #[test]
    fn test_validate_whitelist_allowed() {
        let config = WebSocketConfig {
            allowed_origins: vec!["https://webclient.example.com".to_string()],
            require_origin: true,
            subprotocol: Some("irc".to_string()),
            enable_cors: true,
        };

        let req = mock_request(Some("https://webclient.example.com"), Some("irc"));
        let result = validate_handshake(&req, &config);

        match result {
            HandshakeResult::Accept { .. } => {}
            _ => panic!("Expected Accept for whitelisted origin"),
        }
    }

    #[test]
    fn test_validate_whitelist_rejected() {
        let config = WebSocketConfig {
            allowed_origins: vec!["https://allowed.com".to_string()],
            require_origin: true,
            subprotocol: Some("irc".to_string()),
            enable_cors: true,
        };

        let req = mock_request(Some("https://evil.com"), Some("irc"));
        let result = validate_handshake(&req, &config);

        match result {
            HandshakeResult::Reject { status, reason } => {
                assert_eq!(status, 403);
                assert!(reason.contains("not allowed"));
            }
            _ => panic!("Expected Reject for non-whitelisted origin"),
        }
    }

    #[test]
    fn test_validate_missing_origin_required() {
        let config = WebSocketConfig {
            allowed_origins: Vec::new(),
            require_origin: true,
            subprotocol: Some("irc".to_string()),
            enable_cors: true,
        };

        let req = mock_request(None, Some("irc"));
        let result = validate_handshake(&req, &config);

        match result {
            HandshakeResult::Reject { status, .. } => {
                assert_eq!(status, 403);
            }
            _ => panic!("Expected Reject for missing required Origin"),
        }
    }

    #[test]
    fn test_subprotocol_negotiation() {
        let config = WebSocketConfig {
            allowed_origins: Vec::new(),
            require_origin: false,
            subprotocol: Some("irc".to_string()),
            enable_cors: true,
        };

        let req = mock_request(None, Some("irc, xmpp"));
        let result = validate_handshake(&req, &config);

        match result {
            HandshakeResult::Accept { subprotocol, .. } => {
                assert_eq!(subprotocol, Some("irc".to_string()));
            }
            _ => panic!("Expected Accept with irc subprotocol"),
        }
    }

    #[test]
    fn test_no_subprotocol_negotiation() {
        let config = WebSocketConfig {
            allowed_origins: Vec::new(),
            require_origin: false,
            subprotocol: None,
            enable_cors: true,
        };

        let req = mock_request(None, Some("irc"));
        let result = validate_handshake(&req, &config);

        match result {
            HandshakeResult::Accept { subprotocol, .. } => {
                assert_eq!(subprotocol, None);
            }
            _ => panic!("Expected Accept without subprotocol"),
        }
    }
}
