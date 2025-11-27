use anyhow::Result;
use tokio::net::TcpStream;
use tokio_rustls::server::TlsStream;
use tokio_util::codec::Framed;
use tracing::warn;

use crate::error::ProtocolError;
use crate::irc::IrcCodec;
use crate::Message;
use futures_util::{SinkExt, StreamExt};

#[cfg(feature = "tokio")]
use tokio_tungstenite::{WebSocketStream, tungstenite::Message as WsMessage};

pub const MAX_IRC_LINE_LEN: usize = 8191;

#[derive(Debug)]
pub enum TransportReadError {
    Io(std::io::Error),
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
pub enum Transport {
    Tcp {
        framed: Framed<tokio::net::TcpStream, IrcCodec>,
    },
    Tls {
        framed: Framed<TlsStream<TcpStream>, IrcCodec>,
    },
    #[cfg(feature = "tokio")]
    WebSocket {
        stream: WebSocketStream<TcpStream>,
    },
    #[cfg(feature = "tokio")]
    WebSocketTls {
        stream: WebSocketStream<TlsStream<TcpStream>>,
    },
}

impl Transport {
    pub fn tcp(stream: TcpStream) -> Self {
        if let Err(e) = Self::enable_keepalive(&stream) {
            warn!("failed to enable TCP keepalive: {}", e);
        }

        let codec = IrcCodec::new("utf-8").expect("Failed to create codec");
        Self::Tcp {
            framed: Framed::new(stream, codec),
        }
    }

    fn enable_keepalive(stream: &TcpStream) -> Result<()> {
        use std::time::Duration;
        use socket2::{SockRef, TcpKeepalive};


        let sock = SockRef::from(stream);
        let keepalive = TcpKeepalive::new()
            .with_time(Duration::from_secs(120))
            .with_interval(Duration::from_secs(30));

        sock.set_tcp_keepalive(&keepalive)?;
        Ok(())
    }

    pub fn tls(stream: TlsStream<TcpStream>) -> Self {
        let codec = IrcCodec::new("utf-8").expect("Failed to create codec");
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

        match self {
            Transport::Tcp { framed } => read_framed!(framed),
            Transport::Tls { framed } => read_framed!(framed),
            #[cfg(feature = "tokio")]
            Transport::WebSocket { stream } => {
                let text = read_websocket_message(stream).await?;
                match text {
                    Some(s) => s.parse::<Message>()
                        .map(Some)
                        .map_err(|e| TransportReadError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e))),
                    None => Ok(None)
                }
            }
            #[cfg(feature = "tokio")]
            Transport::WebSocketTls { stream } => {
                let text = read_websocket_message(stream).await?;
                match text {
                    Some(s) => s.parse::<Message>()
                        .map(Some)
                        .map_err(|e| TransportReadError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e))),
                    None => Ok(None)
                }
            }
        }
    }

    pub async fn write_message(&mut self, message: Message) -> Result<()> {
        macro_rules! write_framed {
            ($framed:expr, $msg:expr) => {
                $framed.send($msg).await.map_err(|e| anyhow::anyhow!(e))
            };
        }

        match self {
            Transport::Tcp { framed } => write_framed!(framed, message),
            Transport::Tls { framed } => write_framed!(framed, message),
            #[cfg(feature = "tokio")]
            Transport::WebSocket { stream } => write_websocket_message(stream, &message.to_string()).await,
            #[cfg(feature = "tokio")]
            Transport::WebSocketTls { stream } => write_websocket_message(stream, &message.to_string()).await,
        }
    }
}


#[cfg(feature = "tokio")]
async fn read_websocket_message<S>(stream: &mut WebSocketStream<S>) -> Result<Option<String>, TransportReadError>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    loop {
        match stream.next().await {
            Some(Ok(WsMessage::Text(text))) => {
                if text.len() > MAX_IRC_LINE_LEN {
                    return Err(TransportReadError::Protocol(ProtocolError::MessageTooLong(text.len())));
                }

                let trimmed = text.trim_end_matches(&['\r', '\n'][..]);

                for ch in trimmed.chars() {
                    if ch == '\0' || (ch.is_control() && ch != '\r' && ch != '\n') {
                        return Err(TransportReadError::Protocol(ProtocolError::IllegalControlChar(ch)));
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
    stream.send(WsMessage::Text(msg.to_string())).await
        .map_err(|e| anyhow::anyhow!("WebSocket send error: {}", e))?;
    Ok(())
}

