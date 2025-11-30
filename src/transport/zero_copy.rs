//! Zero-copy transport for high-performance message parsing.
//!
//! This module provides [`ZeroCopyTransport`] which parses IRC messages
//! directly from an internal buffer, yielding borrowed [`MessageRef`] values
//! without heap allocations.

use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::{Buf, BytesMut};
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream as ClientTlsStream;
use tokio_rustls::server::TlsStream as ServerTlsStream;

#[cfg(feature = "tokio")]
use futures_util::{Stream, StreamExt};
#[cfg(feature = "tokio")]
use tokio_tungstenite::{tungstenite::Message as WsMessage, WebSocketStream};

use crate::error::ProtocolError;
use crate::message::MessageRef;

use super::error::TransportReadError;
use super::framed::Transport;
use super::MAX_IRC_LINE_LEN;

// =============================================================================
// LendingStream Trait
// =============================================================================

/// A lending stream trait for zero-copy iteration.
///
/// Unlike `futures::Stream`, this trait allows yielding borrowed data
/// that references the stream's internal buffer. This enables true
/// zero-copy parsing without heap allocations.
///
/// # Generic Associated Types
///
/// This trait uses GATs (Generic Associated Types) to express that the lifetime
/// of yielded items is tied to the borrow of `self`, not to a separate lifetime
/// parameter. GATs were stabilized in Rust 1.65.
///
/// # Stability
///
/// This trait is considered stable for use. The API may evolve in future
/// versions following semver guidelines.
pub trait LendingStream {
    /// The item type yielded by this stream, borrowing from `self`.
    type Item<'a>
    where
        Self: 'a;
    /// The error type that can occur when polling.
    type Error;

    /// Poll the stream for the next item.
    ///
    /// This works similarly to `futures::Stream::poll_next`, but the
    /// returned item borrows from `self`.
    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Item<'_>, Self::Error>>>;
}

// =============================================================================
// ZeroCopyTransport
// =============================================================================

/// Zero-copy transport that yields `MessageRef<'_>` without allocations.
///
/// This transport maintains an internal buffer and parses messages directly
/// from the buffer bytes, yielding borrowed `MessageRef` values that reference
/// the buffer data.
///
/// # Performance
///
/// This transport is designed for hot loops where allocations are expensive:
/// - No heap allocations per message
/// - Minimal buffer management overhead
/// - Direct parsing from byte buffer
///
/// # Usage
///
/// ```ignore
/// let mut transport = ZeroCopyTransport::new(tcp_stream);
/// while let Some(result) = transport.next().await {
///     let msg_ref = result?;
///     // Process msg_ref - it borrows from transport's buffer
/// }
/// ```
pub struct ZeroCopyTransport<S> {
    stream: S,
    buffer: BytesMut,
    consumed: usize,
    max_line_len: usize,
}

impl<S> ZeroCopyTransport<S> {
    /// Create a new zero-copy transport wrapping the given stream.
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            buffer: BytesMut::with_capacity(8192),
            consumed: 0,
            max_line_len: MAX_IRC_LINE_LEN,
        }
    }

    /// Create a new zero-copy transport with an existing buffer.
    ///
    /// This is useful when upgrading from a `Transport` that has buffered
    /// data that hasn't been processed yet.
    pub fn with_buffer(stream: S, buffer: BytesMut) -> Self {
        Self {
            stream,
            buffer,
            consumed: 0,
            max_line_len: MAX_IRC_LINE_LEN,
        }
    }

    /// Create a new zero-copy transport with a custom maximum line length.
    pub fn with_max_line_len(stream: S, max_len: usize) -> Self {
        Self {
            stream,
            buffer: BytesMut::with_capacity(max_len.min(65536)),
            consumed: 0,
            max_line_len: max_len,
        }
    }

    /// Advance the buffer by the consumed amount.
    fn advance_consumed(&mut self) {
        if self.consumed > 0 {
            self.buffer.advance(self.consumed);
            self.consumed = 0;
        }
    }

    /// Find the position of the next line ending (LF) in the buffer.
    fn find_line_end(&self) -> Option<usize> {
        self.buffer.iter().position(|&b| b == b'\n')
    }

    /// Validate a line slice as valid UTF-8 and check for control characters.
    fn validate_line(slice: &[u8]) -> Result<&str, TransportReadError> {
        let s = std::str::from_utf8(slice).map_err(|e| {
            TransportReadError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid UTF-8: {}", e),
            ))
        })?;

        // Trim CRLF for validation
        let trimmed = s.trim_end_matches(['\r', '\n']);

        // Check for NUL and other illegal control characters
        for ch in trimmed.chars() {
            if crate::format::is_illegal_control_char(ch) {
                return Err(TransportReadError::Protocol(
                    ProtocolError::IllegalControlChar(ch),
                ));
            }
        }

        Ok(s)
    }
}

impl<S: AsyncRead + Unpin> ZeroCopyTransport<S> {
    /// Read the next message from the transport.
    ///
    /// Returns `None` when the stream is closed.
    ///
    /// # Example
    ///
    /// ```ignore
    /// while let Some(result) = transport.next().await {
    ///     let msg_ref = result?;
    ///     println!("Command: {}", msg_ref.command_name());
    /// }
    /// ```
    pub async fn next(&mut self) -> Option<Result<MessageRef<'_>, TransportReadError>> {
        // Advance past any previously consumed data
        self.advance_consumed();

        loop {
            // Check if we have a complete line in the buffer
            if let Some(newline_pos) = self.find_line_end() {
                let line_len = newline_pos + 1;

                // Check line length limit
                if line_len > self.max_line_len {
                    return Some(Err(TransportReadError::Protocol(
                        ProtocolError::MessageTooLong {
                            actual: line_len,
                            limit: self.max_line_len,
                        },
                    )));
                }

                // Validate the line
                let line_slice = &self.buffer[..line_len];
                match Self::validate_line(line_slice) {
                    Ok(line_str) => {
                        // Mark this line as consumed (will be advanced on next call)
                        self.consumed = line_len;

                        // Parse the message - no unsafe needed here because:
                        // - The `&mut self` borrow prevents calling `next()` again while MessageRef is live
                        // - Buffer advancement is deferred until the next call to `next()`
                        // - The returned MessageRef lifetime is tied to `self` via function signature
                        match MessageRef::parse(line_str) {
                            Ok(msg) => return Some(Ok(msg)),
                            Err(e) => {
                                return Some(Err(TransportReadError::Protocol(
                                    ProtocolError::InvalidMessage {
                                        string: line_str.to_string(),
                                        cause: e,
                                    },
                                )))
                            }
                        }
                    }
                    Err(e) => return Some(Err(e)),
                }
            }

            // Check if buffer is getting too large without a complete line
            if self.buffer.len() > self.max_line_len {
                return Some(Err(TransportReadError::Protocol(
                    ProtocolError::MessageTooLong {
                        actual: self.buffer.len(),
                        limit: self.max_line_len,
                    },
                )));
            }

            // Need more data - read from stream
            let mut temp = [0u8; 4096];
            match self.stream.read(&mut temp).await {
                Ok(0) => {
                    // EOF - stream closed
                    if self.buffer.is_empty() {
                        return None;
                    } else {
                        // Incomplete message at EOF
                        return Some(Err(TransportReadError::Io(std::io::Error::new(
                            std::io::ErrorKind::UnexpectedEof,
                            "Stream closed with incomplete message",
                        ))));
                    }
                }
                Ok(n) => {
                    self.buffer.extend_from_slice(&temp[..n]);
                }
                Err(e) => return Some(Err(TransportReadError::Io(e))),
            }
        }
    }
}

impl<S: AsyncRead + Unpin> LendingStream for ZeroCopyTransport<S> {
    type Item<'a>
        = MessageRef<'a>
    where
        Self: 'a;
    type Error = TransportReadError;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Item<'_>, Self::Error>>> {
        // Advance past any previously consumed data
        self.advance_consumed();

        loop {
            // Check if we have a complete line in the buffer
            if let Some(newline_pos) = self.find_line_end() {
                let line_len = newline_pos + 1;

                // Check line length limit
                if line_len > self.max_line_len {
                    return Poll::Ready(Some(Err(TransportReadError::Protocol(
                        ProtocolError::MessageTooLong {
                            actual: line_len,
                            limit: self.max_line_len,
                        },
                    ))));
                }

                // Validate the line first (this borrows buffer temporarily)
                {
                    let line_slice = &self.buffer[..line_len];
                    if let Err(e) = Self::validate_line(line_slice) {
                        return Poll::Ready(Some(Err(e)));
                    }
                }

                // Mark this line as consumed
                self.consumed = line_len;

                // Get the line string and parse it.
                //
                // SAFETY: We need to extend the lifetime of the reference to match Self::Item<'_>.
                //
                // This is sound because:
                // 1. The `Pin<&mut Self>` borrow prevents calling `poll_next` again while
                //    the returned MessageRef exists (it would require another &mut borrow)
                // 2. Buffer advancement (`advance_consumed`) is deferred until the next
                //    `poll_next` call, so the data remains valid
                // 3. We don't reallocate or modify the buffer before returning
                //
                // The transmute extends the local borrow's lifetime to match the GAT's
                // `Item<'_>` which is tied to the self borrow via the trait bound `Self: 'a`.
                let line_str: &str = unsafe {
                    let slice = &self.buffer[..line_len];
                    let s = std::str::from_utf8(slice).expect("Already validated as UTF-8");
                    // Extend lifetime from local scope to match Pin<&mut Self>
                    std::mem::transmute::<&str, &str>(s)
                };

                match MessageRef::parse(line_str) {
                    Ok(msg) => return Poll::Ready(Some(Ok(msg))),
                    Err(e) => {
                        return Poll::Ready(Some(Err(TransportReadError::Protocol(
                            ProtocolError::InvalidMessage {
                                string: line_str.to_string(),
                                cause: e,
                            },
                        ))))
                    }
                }
            }

            // Check if buffer is getting too large
            if self.buffer.len() > self.max_line_len {
                return Poll::Ready(Some(Err(TransportReadError::Protocol(
                    ProtocolError::MessageTooLong {
                        actual: self.buffer.len(),
                        limit: self.max_line_len,
                    },
                ))));
            }

            // Need more data - try to read from stream
            let this = self.as_mut().get_mut();
            let mut read_buf = [0u8; 4096];
            let mut read_buf_slice = tokio::io::ReadBuf::new(&mut read_buf);

            match Pin::new(&mut this.stream).poll_read(cx, &mut read_buf_slice) {
                Poll::Ready(Ok(())) => {
                    let n = read_buf_slice.filled().len();
                    if n == 0 {
                        // EOF
                        if this.buffer.is_empty() {
                            return Poll::Ready(None);
                        } else {
                            return Poll::Ready(Some(Err(TransportReadError::Io(
                                std::io::Error::new(
                                    std::io::ErrorKind::UnexpectedEof,
                                    "Stream closed with incomplete message",
                                ),
                            ))));
                        }
                    }
                    this.buffer.extend_from_slice(read_buf_slice.filled());
                    // Loop to check buffer again
                }
                Poll::Ready(Err(e)) => return Poll::Ready(Some(Err(TransportReadError::Io(e)))),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

// =============================================================================
// WebSocket Zero-Copy Transport
// =============================================================================

/// Zero-copy transport wrapper for WebSocket streams.
///
/// WebSocket uses frame-based messaging rather than byte streaming, so this
/// wrapper extracts text payloads from frames and writes them to an internal
/// buffer for zero-copy parsing.
#[cfg(feature = "tokio")]
pub struct ZeroCopyWebSocketTransport<S> {
    stream: WebSocketStream<S>,
    buffer: BytesMut,
    consumed: usize,
    max_line_len: usize,
}

#[cfg(feature = "tokio")]
impl<S> ZeroCopyWebSocketTransport<S> {
    /// Create a new zero-copy WebSocket transport.
    pub fn new(stream: WebSocketStream<S>) -> Self {
        Self {
            stream,
            buffer: BytesMut::with_capacity(8192),
            consumed: 0,
            max_line_len: MAX_IRC_LINE_LEN,
        }
    }

    /// Create with an existing buffer (for upgrade from Transport).
    pub fn with_buffer(stream: WebSocketStream<S>, buffer: BytesMut) -> Self {
        Self {
            stream,
            buffer,
            consumed: 0,
            max_line_len: MAX_IRC_LINE_LEN,
        }
    }

    /// Advance the buffer by the consumed amount.
    fn advance_consumed(&mut self) {
        if self.consumed > 0 {
            self.buffer.advance(self.consumed);
            self.consumed = 0;
        }
    }

    /// Find the position of the next line ending (LF) in the buffer.
    fn find_line_end(&self) -> Option<usize> {
        self.buffer.iter().position(|&b| b == b'\n')
    }

    /// Validate a line slice as valid UTF-8 and check for control characters.
    fn validate_line(slice: &[u8]) -> Result<&str, TransportReadError> {
        let s = std::str::from_utf8(slice).map_err(|e| {
            TransportReadError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid UTF-8: {}", e),
            ))
        })?;

        let trimmed = s.trim_end_matches(['\r', '\n']);

        for ch in trimmed.chars() {
            if crate::format::is_illegal_control_char(ch) {
                return Err(TransportReadError::Protocol(
                    ProtocolError::IllegalControlChar(ch),
                ));
            }
        }

        Ok(s)
    }
}

#[cfg(feature = "tokio")]
impl<S> ZeroCopyWebSocketTransport<S>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    /// Read the next message from the WebSocket transport.
    pub async fn next(&mut self) -> Option<Result<MessageRef<'_>, TransportReadError>> {
        self.advance_consumed();

        loop {
            // Check if we have a complete line in the buffer
            if let Some(newline_pos) = self.find_line_end() {
                let line_len = newline_pos + 1;

                if line_len > self.max_line_len {
                    return Some(Err(TransportReadError::Protocol(
                        ProtocolError::MessageTooLong {
                            actual: line_len,
                            limit: self.max_line_len,
                        },
                    )));
                }

                let line_slice = &self.buffer[..line_len];
                match Self::validate_line(line_slice) {
                    Ok(line_str) => {
                        self.consumed = line_len;

                        match MessageRef::parse(line_str) {
                            Ok(msg) => return Some(Ok(msg)),
                            Err(e) => {
                                return Some(Err(TransportReadError::Protocol(
                                    ProtocolError::InvalidMessage {
                                        string: line_str.to_string(),
                                        cause: e,
                                    },
                                )))
                            }
                        }
                    }
                    Err(e) => return Some(Err(e)),
                }
            }

            // Check buffer size limit
            if self.buffer.len() > self.max_line_len {
                return Some(Err(TransportReadError::Protocol(
                    ProtocolError::MessageTooLong {
                        actual: self.buffer.len(),
                        limit: self.max_line_len,
                    },
                )));
            }

            // Need more data - read from WebSocket
            match self.stream.next().await {
                Some(Ok(WsMessage::Text(text))) => {
                    // WebSocket IRC messages may or may not have CRLF
                    // Append the text, ensuring it ends with LF for our line parser
                    let text = text.trim_end_matches(['\r', '\n']);
                    self.buffer.extend_from_slice(text.as_bytes());
                    self.buffer.extend_from_slice(b"\n");
                }
                Some(Ok(WsMessage::Close(_))) | None => {
                    if self.buffer.is_empty() {
                        return None;
                    } else {
                        return Some(Err(TransportReadError::Io(std::io::Error::new(
                            std::io::ErrorKind::UnexpectedEof,
                            "WebSocket closed with incomplete message",
                        ))));
                    }
                }
                Some(Ok(WsMessage::Ping(_) | WsMessage::Pong(_) | WsMessage::Frame(_))) => {
                    // Ignore control frames, continue reading
                    continue;
                }
                Some(Ok(WsMessage::Binary(_))) => {
                    // IRC is text-only, skip binary frames
                    continue;
                }
                Some(Err(e)) => {
                    return Some(Err(TransportReadError::Io(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("WebSocket error: {}", e),
                    ))));
                }
            }
        }
    }
}

#[cfg(feature = "tokio")]
impl<S> LendingStream for ZeroCopyWebSocketTransport<S>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    type Item<'a>
        = MessageRef<'a>
    where
        Self: 'a;
    type Error = TransportReadError;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Item<'_>, Self::Error>>> {
        self.advance_consumed();

        loop {
            // Check if we have a complete line in the buffer
            if let Some(newline_pos) = self.find_line_end() {
                let line_len = newline_pos + 1;

                if line_len > self.max_line_len {
                    return Poll::Ready(Some(Err(TransportReadError::Protocol(
                        ProtocolError::MessageTooLong {
                            actual: line_len,
                            limit: self.max_line_len,
                        },
                    ))));
                }

                // Validate line first
                {
                    let line_slice = &self.buffer[..line_len];
                    if let Err(e) = Self::validate_line(line_slice) {
                        return Poll::Ready(Some(Err(e)));
                    }
                }

                self.consumed = line_len;

                // SAFETY: Same reasoning as ZeroCopyTransport - Pin<&mut Self> prevents
                // concurrent access, buffer advancement is deferred.
                let line_str: &str = unsafe {
                    let slice = &self.buffer[..line_len];
                    let s = std::str::from_utf8(slice).expect("Already validated as UTF-8");
                    std::mem::transmute::<&str, &str>(s)
                };

                match MessageRef::parse(line_str) {
                    Ok(msg) => return Poll::Ready(Some(Ok(msg))),
                    Err(e) => {
                        return Poll::Ready(Some(Err(TransportReadError::Protocol(
                            ProtocolError::InvalidMessage {
                                string: line_str.to_string(),
                                cause: e,
                            },
                        ))))
                    }
                }
            }

            // Check buffer size limit
            if self.buffer.len() > self.max_line_len {
                return Poll::Ready(Some(Err(TransportReadError::Protocol(
                    ProtocolError::MessageTooLong {
                        actual: self.buffer.len(),
                        limit: self.max_line_len,
                    },
                ))));
            }

            // Need more data - poll WebSocket
            let this = self.as_mut().get_mut();
            match Pin::new(&mut this.stream).poll_next(cx) {
                Poll::Ready(Some(Ok(WsMessage::Text(text)))) => {
                    let text = text.trim_end_matches(['\r', '\n']);
                    this.buffer.extend_from_slice(text.as_bytes());
                    this.buffer.extend_from_slice(b"\n");
                    // Loop to check buffer again
                }
                Poll::Ready(Some(Ok(WsMessage::Close(_)))) | Poll::Ready(None) => {
                    if this.buffer.is_empty() {
                        return Poll::Ready(None);
                    } else {
                        return Poll::Ready(Some(Err(TransportReadError::Io(
                            std::io::Error::new(
                                std::io::ErrorKind::UnexpectedEof,
                                "WebSocket closed with incomplete message",
                            ),
                        ))));
                    }
                }
                Poll::Ready(Some(Ok(
                    WsMessage::Ping(_) | WsMessage::Pong(_) | WsMessage::Frame(_),
                ))) => {
                    continue;
                }
                Poll::Ready(Some(Ok(WsMessage::Binary(_)))) => {
                    continue;
                }
                Poll::Ready(Some(Err(e))) => {
                    return Poll::Ready(Some(Err(TransportReadError::Io(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("WebSocket error: {}", e),
                    )))));
                }
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

// =============================================================================
// ZeroCopyTransportEnum
// =============================================================================

/// Enum wrapper for zero-copy transports over different stream types.
///
/// This provides a unified interface for zero-copy message reading
/// over TCP, TLS, and WebSocket connections.
#[allow(clippy::large_enum_variant)]
#[non_exhaustive]
pub enum ZeroCopyTransportEnum {
    /// TCP zero-copy transport.
    Tcp(ZeroCopyTransport<TcpStream>),
    /// Server-side TLS zero-copy transport.
    Tls(ZeroCopyTransport<ServerTlsStream<TcpStream>>),
    /// Client-side TLS zero-copy transport.
    ClientTls(ZeroCopyTransport<ClientTlsStream<TcpStream>>),
    /// WebSocket zero-copy transport.
    #[cfg(feature = "tokio")]
    WebSocket(ZeroCopyWebSocketTransport<TcpStream>),
    /// WebSocket over TLS zero-copy transport.
    #[cfg(feature = "tokio")]
    WebSocketTls(ZeroCopyWebSocketTransport<ServerTlsStream<TcpStream>>),
}

impl ZeroCopyTransportEnum {
    /// Create a new TCP zero-copy transport.
    pub fn tcp(stream: TcpStream) -> Self {
        Self::Tcp(ZeroCopyTransport::new(stream))
    }

    /// Create a new TCP zero-copy transport with an existing buffer.
    pub fn tcp_with_buffer(stream: TcpStream, buffer: BytesMut) -> Self {
        Self::Tcp(ZeroCopyTransport::with_buffer(stream, buffer))
    }

    /// Create a new server-side TLS zero-copy transport.
    pub fn tls(stream: ServerTlsStream<TcpStream>) -> Self {
        Self::Tls(ZeroCopyTransport::new(stream))
    }

    /// Create a new server-side TLS zero-copy transport with an existing buffer.
    pub fn tls_with_buffer(stream: ServerTlsStream<TcpStream>, buffer: BytesMut) -> Self {
        Self::Tls(ZeroCopyTransport::with_buffer(stream, buffer))
    }

    /// Create a new client-side TLS zero-copy transport.
    pub fn client_tls(stream: ClientTlsStream<TcpStream>) -> Self {
        Self::ClientTls(ZeroCopyTransport::new(stream))
    }

    /// Create a new client-side TLS zero-copy transport with an existing buffer.
    pub fn client_tls_with_buffer(stream: ClientTlsStream<TcpStream>, buffer: BytesMut) -> Self {
        Self::ClientTls(ZeroCopyTransport::with_buffer(stream, buffer))
    }

    /// Create a new WebSocket zero-copy transport.
    #[cfg(feature = "tokio")]
    pub fn websocket(stream: WebSocketStream<TcpStream>) -> Self {
        Self::WebSocket(ZeroCopyWebSocketTransport::new(stream))
    }

    /// Create a new WebSocket zero-copy transport with an existing buffer.
    #[cfg(feature = "tokio")]
    pub fn websocket_with_buffer(stream: WebSocketStream<TcpStream>, buffer: BytesMut) -> Self {
        Self::WebSocket(ZeroCopyWebSocketTransport::with_buffer(stream, buffer))
    }

    /// Create a new WebSocket over TLS zero-copy transport.
    #[cfg(feature = "tokio")]
    pub fn websocket_tls(stream: WebSocketStream<ServerTlsStream<TcpStream>>) -> Self {
        Self::WebSocketTls(ZeroCopyWebSocketTransport::new(stream))
    }

    /// Create a new WebSocket over TLS zero-copy transport with an existing buffer.
    #[cfg(feature = "tokio")]
    pub fn websocket_tls_with_buffer(
        stream: WebSocketStream<ServerTlsStream<TcpStream>>,
        buffer: BytesMut,
    ) -> Self {
        Self::WebSocketTls(ZeroCopyWebSocketTransport::with_buffer(stream, buffer))
    }

    /// Read the next message from the transport.
    pub async fn next(&mut self) -> Option<Result<MessageRef<'_>, TransportReadError>> {
        match self {
            Self::Tcp(t) => t.next().await,
            Self::Tls(t) => t.next().await,
            Self::ClientTls(t) => t.next().await,
            #[cfg(feature = "tokio")]
            Self::WebSocket(t) => t.next().await,
            #[cfg(feature = "tokio")]
            Self::WebSocketTls(t) => t.next().await,
        }
    }
}

impl LendingStream for ZeroCopyTransportEnum {
    type Item<'a>
        = MessageRef<'a>
    where
        Self: 'a;
    type Error = TransportReadError;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Item<'_>, Self::Error>>> {
        match self.get_mut() {
            Self::Tcp(t) => Pin::new(t).poll_next(cx),
            Self::Tls(t) => Pin::new(t).poll_next(cx),
            Self::ClientTls(t) => Pin::new(t).poll_next(cx),
            #[cfg(feature = "tokio")]
            Self::WebSocket(t) => Pin::new(t).poll_next(cx),
            #[cfg(feature = "tokio")]
            Self::WebSocketTls(t) => Pin::new(t).poll_next(cx),
        }
    }
}

// =============================================================================
// From<Transport> for ZeroCopyTransportEnum
// =============================================================================

impl From<Transport> for ZeroCopyTransportEnum {
    /// Convert a `Transport` to a `ZeroCopyTransportEnum`.
    ///
    /// This performs a buffer handover from the `Framed` codec to the
    /// zero-copy transport, ensuring no data is lost during the upgrade.
    ///
    /// All transport types are now supported, including WebSocket.
    fn from(transport: Transport) -> Self {
        match transport {
            Transport::Tcp { framed } => {
                let parts = framed.into_parts();
                ZeroCopyTransportEnum::tcp_with_buffer(parts.io, parts.read_buf)
            }
            Transport::Tls { framed } => {
                let parts = framed.into_parts();
                ZeroCopyTransportEnum::tls_with_buffer(parts.io, parts.read_buf)
            }
            Transport::ClientTls { framed } => {
                let parts = framed.into_parts();
                ZeroCopyTransportEnum::client_tls_with_buffer(parts.io, parts.read_buf)
            }
            #[cfg(feature = "tokio")]
            Transport::WebSocket { stream } => ZeroCopyTransportEnum::websocket(stream),
            #[cfg(feature = "tokio")]
            Transport::WebSocketTls { stream } => ZeroCopyTransportEnum::websocket_tls(stream),
        }
    }
}
