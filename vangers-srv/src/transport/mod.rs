//! Transport layer: handshake and (later) framed I/O setup.

mod handshake;

pub use handshake::{HandshakeError, perform_handshake};
