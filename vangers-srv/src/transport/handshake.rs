//! Vangers TCP handshake: magic string exchange and protocol version negotiation.
//! Performed on a raw `TcpStream` before wrapping with `Framed` and `VangersCodec`.

use ::thiserror::Error;
use ::tokio::io::{AsyncReadExt, AsyncWriteExt};
use ::tokio::net::TcpStream;

/// Client sends this exact string (null-terminated) then protocol version byte.
pub const HS_IN: &[u8] = b"Vivat Sicher, Rock'n'Roll forever!!!";
/// Server responds with this (null-terminated) then protocol version byte.
pub const HS_OUT: &[u8] = b"Enter, my son, please...";

/// Supported protocol versions.
pub const PROTOCOL_VERSIONS: &[u8] = &[1, 2];

#[derive(Error, Debug)]
pub enum HandshakeError {
    #[error("Connection closed by client")]
    ClosedByClient,
    #[error("Handshake: unexpected request header")]
    UnexpectedRequestHeader,
    #[error("Handshake: unexpected protocol version, expected one of: {0:?}, given: {1}")]
    UnexpectedProtocolVersion(&'static [u8], u8),
    #[error("Handshake response fault")]
    ResponseFault,
    #[error("Handshake: zero-terminate symbol is missing in request")]
    ZeroTerminated,
    #[error("Connection fault")]
    Connection,
}

/// Perform Vangers handshake on the given stream.
/// Call this before using `Framed` with `VangersCodec`.
///
/// Returns the negotiated protocol version (1 or 2).
pub async fn perform_handshake(stream: &mut TcpStream) -> Result<u8, HandshakeError> {
    use HandshakeError::*;

    let mut buff = [0u8; 256];

    match stream.read(&mut buff).await {
        Ok(0) => Err(ClosedByClient),
        Ok(_) => {
            if let Some(pos) = buff.iter().position(|&b| b == 0) {
                if !HS_IN.eq(&buff[0..pos]) {
                    return Err(UnexpectedRequestHeader);
                }

                let protocol_version = buff[pos + 1];

                if !matches!(protocol_version, 1 | 2) {
                    return Err(UnexpectedProtocolVersion(
                        PROTOCOL_VERSIONS,
                        protocol_version,
                    ));
                }

                let send: Vec<u8> = HS_OUT
                    .iter()
                    .chain(&[0u8, protocol_version])
                    .copied()
                    .collect();

                if stream.write_all(&send).await.is_err() {
                    return Err(ResponseFault);
                }

                Ok(protocol_version)
            } else {
                Err(ZeroTerminated)
            }
        }
        _ => Err(Connection),
    }
}
