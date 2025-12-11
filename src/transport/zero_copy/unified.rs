//! Unified enum wrapper for all zero-copy transport types.

use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::BytesMut;
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream as ClientTlsStream;
use tokio_rustls::server::TlsStream as ServerTlsStream;

#[cfg(feature = "tokio")]
use tokio_tungstenite::WebSocketStream;

use crate::message::MessageRef;
use crate::Message;

use super::super::error::TransportReadError;
use super::super::framed::Transport;
use super::tcp::ZeroCopyTransport;
use super::trait_def::LendingStream;
use crate::error::ProtocolError;

#[cfg(feature = "tokio")]
use super::websocket::ZeroCopyWebSocketTransport;

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

    /// Write an IRC message to the transport.
    ///
    /// This enables unified read/write operations in a single `tokio::select!`
    /// loop without needing separate writer infrastructure.
    ///
    /// # Example
    ///
    /// ```ignore
    /// loop {
    ///     tokio::select! {
    ///         Some(result) = transport.next() => {
    ///             let msg = result?;
    ///             // handle incoming message
    ///         }
    ///         Some(outgoing) = rx.recv() => {
    ///             transport.write_message(&outgoing).await?;
    ///         }
    ///     }
    /// }
    /// ```
    pub async fn write_message(&mut self, message: &Message) -> std::io::Result<()> {
        match self {
            Self::Tcp(t) => t.write_message(message).await,
            Self::Tls(t) => t.write_message(message).await,
            Self::ClientTls(t) => t.write_message(message).await,
            #[cfg(feature = "tokio")]
            Self::WebSocket(t) => t.write_message(message).await,
            #[cfg(feature = "tokio")]
            Self::WebSocketTls(t) => t.write_message(message).await,
        }
    }

    /// Write a borrowed IRC message to the transport (zero-copy forwarding).
    ///
    /// This is optimized for S2S message forwarding and relay scenarios
    /// where you receive a `MessageRef` and want to forward it without
    /// allocating an owned `Message`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // S2S forwarding: receive from one server, forward to another
    /// while let Some(result) = server_a.next().await {
    ///     let msg_ref = result?;
    ///     if should_forward(&msg_ref) {
    ///         server_b.write_message_ref(&msg_ref).await?;
    ///     }
    /// }
    /// ```
    pub async fn write_message_ref(&mut self, message: &MessageRef<'_>) -> std::io::Result<()> {
        match self {
            Self::Tcp(t) => t.write_message_ref(message).await,
            Self::Tls(t) => t.write_message_ref(message).await,
            Self::ClientTls(t) => t.write_message_ref(message).await,
            #[cfg(feature = "tokio")]
            Self::WebSocket(t) => t.write_message_ref(message).await,
            #[cfg(feature = "tokio")]
            Self::WebSocketTls(t) => t.write_message_ref(message).await,
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

/// Convert a `Transport` to a `ZeroCopyTransportEnum`.
///
/// This performs a buffer handover from the `Framed` codec to the
/// zero-copy transport, ensuring no data is lost during the upgrade.
impl TryFrom<Transport> for ZeroCopyTransportEnum {
    type Error = ProtocolError;

    fn try_from(transport: Transport) -> Result<Self, Self::Error> {
        // Use into_parts() which now supports all transport types including WebSocket.
        let parts = transport
            .into_parts()
            .map_err(|_| ProtocolError::WebSocketNotSupported)?;

        Ok(match parts.stream {
            super::super::parts::TransportStream::Tcp(stream) => {
                ZeroCopyTransportEnum::tcp_with_buffer(stream, parts.read_buf)
            }
            super::super::parts::TransportStream::Tls(stream) => {
                ZeroCopyTransportEnum::tls_with_buffer(*stream, parts.read_buf)
            }
            super::super::parts::TransportStream::ClientTls(stream) => {
                ZeroCopyTransportEnum::client_tls_with_buffer(*stream, parts.read_buf)
            }
            #[cfg(feature = "tokio")]
            super::super::parts::TransportStream::WebSocket(stream) => {
                ZeroCopyTransportEnum::websocket_with_buffer(*stream, parts.read_buf)
            }
            #[cfg(feature = "tokio")]
            super::super::parts::TransportStream::WebSocketTls(stream) => {
                ZeroCopyTransportEnum::websocket_tls_with_buffer(*stream, parts.read_buf)
            }
        })
    }
}
