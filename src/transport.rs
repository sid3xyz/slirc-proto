//! IRC transport layer for async I/O.
//!
//! This module provides transport types for reading and writing IRC messages
//! over TCP, TLS, and WebSocket connections.
//!
//! # Features
//!
//! - [`Transport`]: High-level transport using `Framed` codec for owned [`Message`] types
//! - [`ZeroCopyTransport`]: Zero-allocation transport yielding borrowed [`MessageRef`] types
//! - [`LendingStream`]: Trait for streams that yield borrowed data
//!
//! # Usage
//!
//! Use [`Transport`] during connection handshake and capability negotiation,
//! then upgrade to [`ZeroCopyTransport`] for the hot loop:
//!
//! ```ignore
//! use slirc_proto::transport::{Transport, ZeroCopyTransportEnum};
//!
//! // Use Transport during handshake
//! let transport = Transport::tcp(stream);
//! // ... perform CAP negotiation ...
//!
//! // Upgrade to zero-copy for the hot loop
//! let mut zero_copy: ZeroCopyTransportEnum = transport.try_into()?;
//! while let Some(result) = zero_copy.next().await {
//!     let msg_ref = result?;
//!     // Process MessageRef without allocations
//! }
//!
//! // If you want to split the stream into separate read and write halves while
//! // preserving any bytes that were already read by the framed codec, use
//! // `Transport::into_parts()`:
//! //
//! // ```ignore
//! // // After handshake
//! // let parts = transport.into_parts()?;
//! // // Split into read/write halves
//! // let (read, write) = parts.split();
//! // // Seed the zero-copy reader with leftover bytes
//! // let mut zero_copy = ZeroCopyTransport::with_buffer(read.half, read.read_buf);
//! // // Create a framed writer using the write half and codec
//! // let mut writer = tokio_util::codec::FramedWrite::new(write.half, write.codec);
//! // ```
//! ```
//!
//! [`Message`]: crate::Message
//! [`MessageRef`]: crate::MessageRef

use std::pin::Pin;
use std::task::{Context, Poll};

use anyhow::Result;
use bytes::{Buf, BytesMut};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite};
use tokio::net::TcpStream;
use tokio_rustls::server::TlsStream;
use tokio_util::codec::Framed;
use tracing::warn;

use crate::error::ProtocolError;
use crate::irc::IrcCodec;
use crate::message::MessageRef;
use crate::Message;
use futures_util::{SinkExt, StreamExt};

#[cfg(feature = "tokio")]
use tokio_tungstenite::{tungstenite::Message as WsMessage, WebSocketStream};

/// Maximum IRC line length (8191 bytes as per modern IRC conventions).
pub const MAX_IRC_LINE_LEN: usize = 8191;

/// Errors that can occur when reading from a transport.
#[derive(Debug)]
#[non_exhaustive]
pub enum TransportReadError {
    /// An I/O error occurred.
    Io(std::io::Error),
    /// A protocol error occurred.
    Protocol(ProtocolError),
}

impl From<std::io::Error> for TransportReadError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<ProtocolError> for TransportReadError {
    fn from(err: ProtocolError) -> Self {
        Self::Protocol(err)
    }
}

#[allow(clippy::large_enum_variant)]
#[non_exhaustive]
pub enum Transport {
    Tcp {
        framed: Framed<tokio::net::TcpStream, IrcCodec>,
    },
    Tls {
        framed: Framed<TlsStream<TcpStream>, IrcCodec>,
    },
    #[cfg(feature = "tokio")]
    WebSocket { stream: WebSocketStream<TcpStream> },
    #[cfg(feature = "tokio")]
    WebSocketTls {
        stream: WebSocketStream<TlsStream<TcpStream>>,
    },
}

/// A unified raw transport stream type for hand-off to users.
#[non_exhaustive]
pub enum TransportStream {
    Tcp(TcpStream),
    Tls(Box<TlsStream<TcpStream>>),
    #[cfg(feature = "tokio")]
    WebSocket(Box<WebSocketStream<TcpStream>>),
    #[cfg(feature = "tokio")]
    WebSocketTls(Box<WebSocketStream<TlsStream<TcpStream>>>),
}

/// The parts extracted from a `Transport`, including any buffered data
/// that has already been read but not yet parsed.
pub struct TransportParts {
    pub stream: TransportStream,
    pub read_buf: BytesMut,
    pub write_buf: BytesMut,
    // Keep the codec so it can be used to re-create framed writers if needed
    pub codec: IrcCodec,
}

/// Owned read half for a transport after splitting.
pub enum TransportReadHalf {
    Tcp(tokio::net::tcp::OwnedReadHalf),
    Tls(tokio::io::ReadHalf<TlsStream<TcpStream>>),
}

/// Owned write half for a transport after splitting.
pub enum TransportWriteHalf {
    Tcp(tokio::net::tcp::OwnedWriteHalf),
    Tls(tokio::io::WriteHalf<TlsStream<TcpStream>>),
}

impl AsyncRead for TransportReadHalf {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            Self::Tcp(inner) => Pin::new(inner).poll_read(cx, buf),
            Self::Tls(inner) => Pin::new(inner).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for TransportWriteHalf {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        match self.get_mut() {
            Self::Tcp(inner) => Pin::new(inner).poll_write(cx, buf),
            Self::Tls(inner) => Pin::new(inner).poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            Self::Tcp(inner) => Pin::new(inner).poll_flush(cx),
            Self::Tls(inner) => Pin::new(inner).poll_flush(cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            Self::Tcp(inner) => Pin::new(inner).poll_shutdown(cx),
            Self::Tls(inner) => Pin::new(inner).poll_shutdown(cx),
        }
    }
}

/// A convenience container for a split transport read side with any pre-seeded
/// buffer loaded from the original framed transport.
pub struct TransportRead {
    pub half: TransportReadHalf,
    pub read_buf: BytesMut,
}

/// A convenience container for a split transport write side including any
/// write buffer and codec to reconstruct a framed writer.
pub struct TransportWrite {
    pub half: TransportWriteHalf,
    pub write_buf: BytesMut,
    pub codec: IrcCodec,
}

impl TransportParts {
    /// Split the `TransportParts` into read & write halves suitable for
    /// spawning separate tasks. The read half contains any leftover bytes
    /// that were read but not yet parsed; the write half contains the
    /// codec and write buffer allowing the caller to create a framed sink.
    pub fn split(self) -> (TransportRead, TransportWrite) {
        match self.stream {
            TransportStream::Tcp(stream) => {
                let (r, w) = stream.into_split();
                (
                    TransportRead {
                        half: TransportReadHalf::Tcp(r),
                        read_buf: self.read_buf,
                    },
                    TransportWrite {
                        half: TransportWriteHalf::Tcp(w),
                        write_buf: self.write_buf,
                        codec: self.codec,
                    },
                )
            }
            TransportStream::Tls(stream) => {
                // Unbox and split the TLS stream
                let (r, w) = tokio::io::split(*stream);
                (
                    TransportRead {
                        half: TransportReadHalf::Tls(r),
                        read_buf: self.read_buf,
                    },
                    TransportWrite {
                        half: TransportWriteHalf::Tls(w),
                        write_buf: self.write_buf,
                        codec: self.codec,
                    },
                )
            }
            #[cfg(feature = "tokio")]
            TransportStream::WebSocket(_ws) => {
                // WebSocket streams don't have a sensible split that maintains
                // the line-based message semantics; return sink/stream halves.
                // We intentionally panic here to make unsupported usage explicit.
                panic!("WebSocket split not supported via TransportParts::split");
            }
            #[cfg(feature = "tokio")]
            TransportStream::WebSocketTls(_ws) => {
                panic!("WebSocketTls split not supported via TransportParts::split");
            }
        }
    }
}

impl Transport {
    pub fn tcp(stream: TcpStream) -> Self {
        if let Err(e) = Self::enable_keepalive(&stream) {
            warn!("failed to enable TCP keepalive: {}", e);
        }

        let codec =
            IrcCodec::new("utf-8").expect("Failed to create UTF-8 codec: encoding not supported");
        Self::Tcp {
            framed: Framed::new(stream, codec),
        }
    }

    fn enable_keepalive(stream: &TcpStream) -> Result<()> {
        use socket2::{SockRef, TcpKeepalive};
        use std::time::Duration;

        let sock = SockRef::from(stream);
        let keepalive = TcpKeepalive::new()
            .with_time(Duration::from_secs(120))
            .with_interval(Duration::from_secs(30));

        sock.set_tcp_keepalive(&keepalive)?;
        Ok(())
    }

    pub fn tls(stream: TlsStream<TcpStream>) -> Self {
        let codec =
            IrcCodec::new("utf-8").expect("Failed to create UTF-8 codec: encoding not supported");
        Self::Tls {
            framed: Framed::new(stream, codec),
        }
    }

    #[cfg(feature = "tokio")]
    pub fn websocket(stream: WebSocketStream<TcpStream>) -> Self {
        Self::WebSocket { stream }
    }

    #[cfg(feature = "tokio")]
    pub fn websocket_tls(stream: WebSocketStream<TlsStream<TcpStream>>) -> Self {
        Self::WebSocketTls { stream }
    }

    /// Consume the `Transport`, returning the underlying raw stream and any
    /// buffered bytes that were read but not yet parsed. This is intended for
    /// callers that want to take over I/O (for example to spawn a writer task)
    /// while preserving any buffered data for a zero-copy reader.
    pub fn into_parts(self) -> Result<TransportParts, WebSocketNotSupportedError> {
        match self {
            Transport::Tcp { framed } => {
                let parts = framed.into_parts();
                Ok(TransportParts {
                    stream: TransportStream::Tcp(parts.io),
                    read_buf: parts.read_buf,
                    write_buf: parts.write_buf,
                    codec: parts.codec,
                })
            }
            Transport::Tls { framed } => {
                let parts = framed.into_parts();
                Ok(TransportParts {
                    stream: TransportStream::Tls(Box::new(parts.io)),
                    read_buf: parts.read_buf,
                    write_buf: parts.write_buf,
                    codec: parts.codec,
                })
            }
            #[cfg(feature = "tokio")]
            _ => Err(WebSocketNotSupportedError),
        }
    }

    pub fn is_tls(&self) -> bool {
        matches!(self, Self::Tls { .. })
    }

    pub fn is_websocket(&self) -> bool {
        #[cfg(feature = "tokio")]
        {
            matches!(self, Self::WebSocket { .. } | Self::WebSocketTls { .. })
        }
        #[cfg(not(feature = "tokio"))]
        {
            false
        }
    }

    pub async fn read_message(&mut self) -> Result<Option<Message>, TransportReadError> {
        macro_rules! read_framed {
            ($framed:expr) => {
                match $framed.next().await {
                    Some(Ok(msg)) => Ok(Some(msg)),
                    Some(Err(e)) => Err(TransportReadError::from(e)),
                    None => Ok(None),
                }
            };
        }

        macro_rules! read_websocket {
            ($stream:expr) => {{
                let text = read_websocket_message($stream).await?;
                match text {
                    Some(s) => s
                        .parse::<Message>()
                        .map(Some)
                        .map_err(TransportReadError::from),
                    None => Ok(None),
                }
            }};
        }

        match self {
            Transport::Tcp { framed } => read_framed!(framed),
            Transport::Tls { framed } => read_framed!(framed),
            #[cfg(feature = "tokio")]
            Transport::WebSocket { stream } => read_websocket!(stream),
            #[cfg(feature = "tokio")]
            Transport::WebSocketTls { stream } => read_websocket!(stream),
        }
    }

    pub async fn write_message(&mut self, message: &Message) -> Result<()> {
        macro_rules! write_framed {
            ($framed:expr, $msg:expr) => {
                $framed
                    .send($msg.clone())
                    .await
                    .map_err(|e| anyhow::anyhow!(e))
            };
        }

        match self {
            Transport::Tcp { framed } => write_framed!(framed, message),
            Transport::Tls { framed } => write_framed!(framed, message),
            #[cfg(feature = "tokio")]
            Transport::WebSocket { stream } => {
                write_websocket_message(stream, &message.to_string()).await
            }
            #[cfg(feature = "tokio")]
            Transport::WebSocketTls { stream } => {
                write_websocket_message(stream, &message.to_string()).await
            }
        }
    }
}

#[cfg(feature = "tokio")]
async fn read_websocket_message<S>(
    stream: &mut WebSocketStream<S>,
) -> Result<Option<String>, TransportReadError>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    loop {
        match stream.next().await {
            Some(Ok(WsMessage::Text(text))) => {
                if text.len() > MAX_IRC_LINE_LEN {
                    return Err(TransportReadError::Protocol(ProtocolError::MessageTooLong(
                        text.len(),
                    )));
                }

                let trimmed = text.trim_end_matches(&['\r', '\n'][..]);

                for ch in trimmed.chars() {
                    if ch == '\0' || (ch.is_control() && ch != '\r' && ch != '\n') {
                        return Err(TransportReadError::Protocol(
                            ProtocolError::IllegalControlChar(ch),
                        ));
                    }
                }

                return Ok(Some(trimmed.to_string()));
            }
            Some(Ok(WsMessage::Close(_))) | None => {
                return Ok(None);
            }
            Some(Ok(WsMessage::Ping(_))) | Some(Ok(WsMessage::Pong(_))) => {
                continue;
            }
            Some(Ok(WsMessage::Binary(_))) => {
                warn!("Ignoring binary WebSocket frame (IRC is text-only)");
                continue;
            }
            Some(Ok(WsMessage::Frame(_))) => {
                continue;
            }
            Some(Err(e)) => {
                return Err(TransportReadError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("WebSocket error: {}", e),
                )));
            }
        }
    }
}

#[cfg(feature = "tokio")]
async fn write_websocket_message<S>(stream: &mut WebSocketStream<S>, message: &str) -> Result<()>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let msg = message.trim_end_matches(&['\r', '\n'][..]);
    stream
        .send(WsMessage::Text(msg.to_string()))
        .await
        .map_err(|e| anyhow::anyhow!("WebSocket send error: {}", e))?;
    Ok(())
}

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
            if ch == '\0' || (ch.is_control() && ch != '\r' && ch != '\n') {
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
                        ProtocolError::MessageTooLong(line_len),
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
                    ProtocolError::MessageTooLong(self.buffer.len()),
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
                        ProtocolError::MessageTooLong(line_len),
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
                    ProtocolError::MessageTooLong(self.buffer.len()),
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
// ZeroCopyTransportEnum
// =============================================================================

/// Enum wrapper for zero-copy transports over different stream types.
///
/// This provides a unified interface for zero-copy message reading
/// over TCP and TLS connections.
#[allow(clippy::large_enum_variant)]
#[non_exhaustive]
pub enum ZeroCopyTransportEnum {
    /// TCP zero-copy transport.
    Tcp(ZeroCopyTransport<TcpStream>),
    /// TLS zero-copy transport.
    Tls(ZeroCopyTransport<TlsStream<TcpStream>>),
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

    /// Create a new TLS zero-copy transport.
    pub fn tls(stream: TlsStream<TcpStream>) -> Self {
        Self::Tls(ZeroCopyTransport::new(stream))
    }

    /// Create a new TLS zero-copy transport with an existing buffer.
    pub fn tls_with_buffer(stream: TlsStream<TcpStream>, buffer: BytesMut) -> Self {
        Self::Tls(ZeroCopyTransport::with_buffer(stream, buffer))
    }

    /// Read the next message from the transport.
    pub async fn next(&mut self) -> Option<Result<MessageRef<'_>, TransportReadError>> {
        match self {
            Self::Tcp(t) => t.next().await,
            Self::Tls(t) => t.next().await,
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
        }
    }
}

// =============================================================================
// TryFrom<Transport> for ZeroCopyTransportEnum
// =============================================================================

/// Error returned when converting a WebSocket transport to zero-copy.
///
/// WebSocket transports cannot be converted to zero-copy because the
/// WebSocket framing protocol requires different handling.
pub struct WebSocketNotSupportedError;

impl std::fmt::Debug for WebSocketNotSupportedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "WebSocket transport cannot be converted to zero-copy transport"
        )
    }
}

impl std::fmt::Display for WebSocketNotSupportedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "WebSocket transport cannot be converted to zero-copy transport"
        )
    }
}

impl std::error::Error for WebSocketNotSupportedError {}

impl TryFrom<Transport> for ZeroCopyTransportEnum {
    type Error = WebSocketNotSupportedError;

    /// Convert a `Transport` to a `ZeroCopyTransportEnum`.
    ///
    /// This performs a buffer handover from the `Framed` codec to the
    /// zero-copy transport, ensuring no data is lost during the upgrade.
    ///
    /// # Errors
    ///
    /// Returns `Err(WebSocketNotSupportedError)` if the transport is a
    /// WebSocket variant, as WebSocket requires different framing.
    fn try_from(transport: Transport) -> Result<Self, Self::Error> {
        let parts = transport.into_parts()?;
        match parts.stream {
            TransportStream::Tcp(io) => {
                Ok(ZeroCopyTransportEnum::tcp_with_buffer(io, parts.read_buf))
            }
            TransportStream::Tls(io) => {
                Ok(ZeroCopyTransportEnum::tls_with_buffer(*io, parts.read_buf))
            }
            #[cfg(feature = "tokio")]
            _ => unreachable!("WebSocket transports cannot be converted to zero-copy"),
            // WebSocket variants are already handled by `into_parts` returning
            // a `WebSocketNotSupportedError` earlier, so we don't handle them
            // here.
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;
    use std::io::Cursor;
    use tokio::io::AsyncRead;

    /// A mock async reader that returns data from a byte slice.
    struct MockReader {
        data: Cursor<Vec<u8>>,
    }

    impl MockReader {
        fn new(data: &[u8]) -> Self {
            Self {
                data: Cursor::new(data.to_vec()),
            }
        }
    }

    impl AsyncRead for MockReader {
        fn poll_read(
            mut self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            buf: &mut tokio::io::ReadBuf<'_>,
        ) -> Poll<std::io::Result<()>> {
            let pos = self.data.position() as usize;
            let data = self.data.get_ref();
            if pos >= data.len() {
                return Poll::Ready(Ok(()));
            }
            let to_read = (data.len() - pos).min(buf.remaining());
            buf.put_slice(&data[pos..pos + to_read]);
            self.data.set_position((pos + to_read) as u64);
            Poll::Ready(Ok(()))
        }
    }

    impl Unpin for MockReader {}

    #[tokio::test]
    async fn test_zero_copy_simple() {
        let data = b"PING :server\r\n";
        let reader = MockReader::new(data);
        let mut transport = ZeroCopyTransport::new(reader);

        let result = transport.next().await;
        assert!(result.is_some());
        let msg = result.unwrap().unwrap();
        assert_eq!(msg.command_name(), "PING");
        assert_eq!(msg.args(), &["server"]);
    }

    #[tokio::test]
    async fn test_zero_copy_multiple_messages() {
        let data = b"PING :server1\r\nPING :server2\r\n";
        let reader = MockReader::new(data);
        let mut transport = ZeroCopyTransport::new(reader);

        let msg1 = transport.next().await.unwrap().unwrap();
        assert_eq!(msg1.args(), &["server1"]);

        let msg2 = transport.next().await.unwrap().unwrap();
        assert_eq!(msg2.args(), &["server2"]);

        let msg3 = transport.next().await;
        assert!(msg3.is_none());
    }

    #[tokio::test]
    async fn test_zero_copy_with_tags() {
        let data = b"@time=2023-01-01;msgid=abc :nick!user@host PRIVMSG #channel :Hello\r\n";
        let reader = MockReader::new(data);
        let mut transport = ZeroCopyTransport::new(reader);

        let msg = transport.next().await.unwrap().unwrap();
        assert_eq!(msg.command_name(), "PRIVMSG");
        assert_eq!(msg.tag_value("time"), Some("2023-01-01"));
        assert_eq!(msg.tag_value("msgid"), Some("abc"));
        assert_eq!(msg.source_nickname(), Some("nick"));
    }

    #[tokio::test]
    async fn test_zero_copy_oversized() {
        // Create a line that exceeds the max length
        let long_line = format!("PRIVMSG #channel :{}\r\n", "A".repeat(MAX_IRC_LINE_LEN));
        let reader = MockReader::new(long_line.as_bytes());
        let mut transport = ZeroCopyTransport::new(reader);

        let result = transport.next().await;
        assert!(result.is_some());
        match result.unwrap() {
            Err(TransportReadError::Protocol(ProtocolError::MessageTooLong(_))) => {}
            other => panic!("Expected MessageTooLong error, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_zero_copy_with_buffer() {
        // Simulate upgrading from Transport with buffered data
        let mut buffer = BytesMut::new();
        buffer.extend_from_slice(b"PING :buffered\r\n");

        let reader = MockReader::new(b"PING :fresh\r\n");
        let mut transport = ZeroCopyTransport::with_buffer(reader, buffer);

        // Should get buffered message first
        let msg1 = transport.next().await.unwrap().unwrap();
        assert_eq!(msg1.args(), &["buffered"]);

        // Then fresh data
        let msg2 = transport.next().await.unwrap().unwrap();
        assert_eq!(msg2.args(), &["fresh"]);
    }

    #[tokio::test]
    async fn test_zero_copy_lf_only() {
        // IRC also accepts LF without CR
        let data = b"PING :server\n";
        let reader = MockReader::new(data);
        let mut transport = ZeroCopyTransport::new(reader);

        let msg = transport.next().await.unwrap().unwrap();
        assert_eq!(msg.command_name(), "PING");
    }

    #[tokio::test]
    async fn test_zero_copy_invalid_utf8() {
        let data = [b'P', b'I', b'N', b'G', b' ', 0xFF, 0xFE, b'\r', b'\n'];
        let reader = MockReader::new(&data);
        let mut transport = ZeroCopyTransport::new(reader);

        let result = transport.next().await;
        assert!(result.is_some());
        match result.unwrap() {
            Err(TransportReadError::Io(e)) => {
                assert!(e.to_string().contains("UTF-8"));
            }
            other => panic!("Expected UTF-8 error, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_zero_copy_control_char() {
        // NUL character should be rejected
        let data = b"PING :server\x00test\r\n";
        let reader = MockReader::new(data);
        let mut transport = ZeroCopyTransport::new(reader);

        let result = transport.next().await;
        assert!(result.is_some());
        match result.unwrap() {
            Err(TransportReadError::Protocol(ProtocolError::IllegalControlChar('\0'))) => {}
            other => panic!("Expected IllegalControlChar error, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_zero_copy_fragmented() {
        // Simulate data arriving in small chunks
        // For this test, we use a reader that gives all data at once,
        // but we verify parsing works correctly with various message types
        let data = b":server 001 nick :Welcome\r\n:server 002 nick :Your host\r\n";
        let reader = MockReader::new(data);
        let mut transport = ZeroCopyTransport::new(reader);

        let msg1 = transport.next().await.unwrap().unwrap();
        assert!(msg1.is_numeric());
        assert_eq!(msg1.numeric_code(), Some(1));

        let msg2 = transport.next().await.unwrap().unwrap();
        assert!(msg2.is_numeric());
        assert_eq!(msg2.numeric_code(), Some(2));

        assert!(transport.next().await.is_none());
    }

    #[tokio::test]
    async fn test_zero_copy_eof_incomplete() {
        // Data with no newline - should error on EOF
        let data = b"PING :incomplete";
        let reader = MockReader::new(data);
        let mut transport = ZeroCopyTransport::new(reader);

        let result = transport.next().await;
        assert!(result.is_some());
        match result.unwrap() {
            Err(TransportReadError::Io(e)) => {
                assert_eq!(e.kind(), std::io::ErrorKind::UnexpectedEof);
            }
            other => panic!("Expected UnexpectedEof error, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_into_parts_preserves_buffer() {
        use tokio::io::AsyncWriteExt;
        use tokio::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let client = async move {
            let mut s = tokio::net::TcpStream::connect(addr).await.unwrap();
            // Send both messages in a single write so the framed reader may read
            // both and leave the second in the read buffer.
            s.write_all(b"NICK test\r\nUSER test 0 * :Test\r\n")
                .await
                .unwrap();
        };

        let server = async move {
            let (stream, _peer) = listener.accept().await.unwrap();
            let mut transport = Transport::tcp(stream);

            let msg = transport.read_message().await.unwrap().unwrap();
            use crate::command::Command;
            match msg.command {
                Command::NICK(_) => {}
                _ => panic!("Expected NICK command"),
            }

            let parts = transport.into_parts().unwrap();
            // Ensure there is leftover data with USER
            let leftover = std::str::from_utf8(&parts.read_buf).unwrap();
            assert!(leftover.contains("USER "));
        };

        tokio::join!(client, server);
    }

    #[tokio::test]
    async fn test_upgrade_split_zero_copy() {
        use crate::command::Command;
        use crate::message::Message;
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpListener;
        use tokio_util::codec::FramedWrite;

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let client = async move {
            let mut s = tokio::net::TcpStream::connect(addr).await.unwrap();
            s.write_all(b"NICK test\r\nUSER test 0 * :Test\r\n")
                .await
                .unwrap();

            // Read response from server (writer) - the server will send a PRIVMSG
            let mut buf = [0u8; 1024];
            let n = s.read(&mut buf).await.unwrap();
            let s = std::str::from_utf8(&buf[..n]).unwrap();
            assert!(s.contains("PRIVMSG"));
        };

        let server = async move {
            let (stream, _peer) = listener.accept().await.unwrap();
            let mut transport = Transport::tcp(stream);

            // Read first message (NICK)
            let msg = transport.read_message().await.unwrap().unwrap();
            match msg.command {
                Command::NICK(_) => {}
                _ => panic!("Expected NICK command"),
            }

            // Upgrade & split
            let parts = transport.into_parts().unwrap();
            let (read, write) = parts.split();

            // Create zero-copy reader using the read half and read buffer
            match read.half {
                TransportReadHalf::Tcp(r) => {
                    let mut zero = ZeroCopyTransport::with_buffer(r, read.read_buf);
                    // For the purposes of this test, read next message (USER)
                    let next_msg = zero.next().await.unwrap().unwrap();
                    assert!(next_msg.is_numeric() || next_msg.command_name() != "");
                }
                _ => panic!("Expected Tcp read half"),
            }

            // For writer, send a PRIVMSG to the client
            match write.half {
                TransportWriteHalf::Tcp(w) => {
                    let mut framed_write = FramedWrite::new(w, write.codec);
                    framed_write
                        .send(Message::privmsg("test", "Hello from server"))
                        .await
                        .unwrap();
                }
                _ => panic!("Expected Tcp write half"),
            }
        };

        tokio::join!(client, server);
    }
}
