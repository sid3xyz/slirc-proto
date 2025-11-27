
use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio_rustls::server::TlsStream;
use tracing::warn;

#[cfg(feature = "tokio")]
use tokio_tungstenite::{WebSocketStream, tungstenite::Message as WsMessage};
#[cfg(feature = "tokio")]
use futures_util::{SinkExt, StreamExt};

pub const MAX_IRC_LINE_LEN: usize = 8191;

const MAX_LINE_PREVIEW_LEN: usize = 512;

#[derive(Debug)]
pub enum TransportReadError {
    Io(std::io::Error),
    LineTooLong {
        preview: String
    },
    IllegalControlChar {
        ch: char,
        preview: String,
    },
}

impl From<std::io::Error> for TransportReadError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

#[allow(clippy::large_enum_variant)]
pub enum Transport {
    Tcp {
        reader: BufReader<tokio::net::tcp::OwnedReadHalf>,
        writer: tokio::net::tcp::OwnedWriteHalf,
    },
    Tls {
        reader: BufReader<tokio::io::ReadHalf<TlsStream<TcpStream>>>,
        writer: tokio::io::WriteHalf<TlsStream<TcpStream>>,
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

        let (read, write) = stream.into_split();
        Self::Tcp {
            reader: BufReader::new(read),
            writer: write,
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
        let (read, write) = tokio::io::split(stream);
        Self::Tls {
            reader: BufReader::new(read),
            writer: write,
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

    pub async fn read_message(&mut self) -> Result<Option<String>, TransportReadError> {
        match self {
            Transport::Tcp { reader, .. } => read_line_limited(reader).await,
            Transport::Tls { reader, .. } => read_line_limited(reader).await,
            #[cfg(feature = "tokio")]
            Transport::WebSocket { stream } => read_websocket_message(stream).await,
            #[cfg(feature = "tokio")]
            Transport::WebSocketTls { stream } => read_websocket_message(stream).await,
        }
    }

    pub async fn write_message(&mut self, message: &str) -> Result<()> {
        match self {
            Transport::Tcp { writer, .. } => {
                writer.write_all(message.as_bytes()).await?;
                writer.flush().await?;
                Ok(())
            }
            Transport::Tls { writer, .. } => {
                writer.write_all(message.as_bytes()).await?;
                writer.flush().await?;
                Ok(())
            }
            #[cfg(feature = "tokio")]
            Transport::WebSocket { stream } => write_websocket_message(stream, message).await,
            #[cfg(feature = "tokio")]
            Transport::WebSocketTls { stream } => write_websocket_message(stream, message).await,
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
                    let preview = text.chars().take(MAX_LINE_PREVIEW_LEN).collect();
                    return Err(TransportReadError::LineTooLong { preview });
                }


                let trimmed = text.trim_end_matches(&['\r', '\n'][..]);


                for ch in trimmed.chars() {
                    if ch == '\0' || (ch.is_control() && ch != '\r' && ch != '\n') {
                        let preview = trimmed.chars().take(MAX_LINE_PREVIEW_LEN).collect();
                        return Err(TransportReadError::IllegalControlChar { ch, preview });
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

async fn read_line_limited<R>(reader: &mut BufReader<R>) -> Result<Option<String>, TransportReadError>
where
    R: AsyncRead + Unpin,
{
    let mut line: Vec<u8> = Vec::with_capacity(512);
    let mut exceeded_limit = false;

    loop {
        let buffer = reader.fill_buf().await?;

        if buffer.is_empty() {
            if line.is_empty() && !exceeded_limit {
                return Ok(None);
            }
            break;
        }

        let newline_pos = buffer.iter().position(|&b| b == b'\n');
        let to_consume = newline_pos.map_or(buffer.len(), |idx| idx + 1);

        if !exceeded_limit {
            let projected_len = line.len().saturating_add(to_consume);
            if projected_len > MAX_IRC_LINE_LEN {
                let available = MAX_IRC_LINE_LEN.saturating_sub(line.len());
                line.extend_from_slice(&buffer[..available.min(buffer.len())]);
                exceeded_limit = true;
            } else {
                line.extend_from_slice(&buffer[..to_consume]);
            }
        }

        reader.consume(to_consume);

        if newline_pos.is_some() {
            break;
        }
    }

    if exceeded_limit {
        warn!(
            length = line.len(),
            "Message exceeds {} byte limit",
            MAX_IRC_LINE_LEN
        );

        let preview_len = line.len().min(MAX_LINE_PREVIEW_LEN);
        let preview = String::from_utf8_lossy(&line[..preview_len]).to_string();
        return Err(TransportReadError::LineTooLong { preview });
    }

    while matches!(line.last(), Some(b'\r') | Some(b'\n')) {
        line.pop();
    }

    if line.is_empty() {
        Ok(Some(String::new()))
    } else {
        let line_str = String::from_utf8_lossy(&line).to_string();



        for ch in line_str.chars() {



            if ch == '\0' || (ch.is_control() && ch != '\r' && ch != '\n' && ch != '\u{0001}') {
                let preview = line_str.chars()
                    .take(MAX_LINE_PREVIEW_LEN)
                    .collect();
                return Err(TransportReadError::IllegalControlChar { ch, preview });
            }
        }

        Ok(Some(line_str))
    }
}

