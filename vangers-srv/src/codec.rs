//! Vangers protocol framing: length-prefixed packets (2 bytes LE size + 1 byte action + payload).
//! Used with `tokio_util::codec::Framed` to turn a byte stream into a stream/sink of `Packet`.

use std::io;

use ::bytes::BufMut;
use ::tokio_util::codec::{Decoder, Encoder};

use crate::protocol::Packet;

/// Codec for Vangers binary protocol over TCP.
///
/// Frame format: `[event_size: i16 LE][action: u8][data: ..]`
/// where `event_size` = 1 + data.len() (i.e. action byte + payload length).
/// Total frame length = 2 + event_size bytes.
#[derive(Debug, Default, Clone, Copy)]
pub struct VangersCodec;

impl Decoder for VangersCodec {
    type Item = Packet;
    type Error = io::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Packet>, Self::Error> {
        if src.len() < 2 {
            return Ok(None);
        }

        let event_size = i16::from_le_bytes([src[0], src[1]]);
        if event_size < 0 {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid event_size: {}", event_size),
            ))?
        }

        let total = 2 + event_size as usize;
        if src.len() < total {
            src.reserve(total.saturating_sub(src.len()));
            return Ok(None);
        }

        let frame = src.split_to(total);
        let packet = Packet::from_slice(&frame);
        Ok(Some(packet))
    }
}

impl Encoder<Packet> for VangersCodec {
    type Error = io::Error;

    fn encode(&mut self, item: Packet, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        let bytes = item.as_bytes();
        dst.reserve(bytes.len());
        dst.put_slice(&bytes);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::Action;

    #[test]
    fn decode_encode_roundtrip() {
        let packet = Packet::new(Action::SERVER_TIME_QUERY, &[1, 2, 3]).as_bytes();
        let mut buf = bytes::BytesMut::from(&packet[..]);
        let decoded = VangersCodec.decode(&mut buf).unwrap().unwrap();
        assert_eq!(decoded.action, Action::SERVER_TIME_QUERY);
        assert_eq!(decoded.data, &[1, 2, 3]);

        let mut out = bytes::BytesMut::new();
        VangersCodec.encode(decoded, &mut out).unwrap();
        assert_eq!(out.to_vec(), packet);
    }

    #[test]
    fn decode_returns_none_until_full_frame() {
        // save to `buf` only 2 bytes, so this is not a full frame
        let mut buf = bytes::BytesMut::from(&[1u8, 0][..]);
        assert!(VangersCodec.decode(&mut buf).unwrap().is_none());

        // frame finished
        buf.put_u8(0x89);
        let p = VangersCodec.decode(&mut buf).unwrap().unwrap();
        assert_eq!(p.action, Action::SERVER_TIME_QUERY);
        assert!(buf.is_empty());
    }

    #[test]
    fn decode_returns_packet_with_trash_bytes_at_the_buffer_end() {
        let mut buf = bytes::BytesMut::from(&[1u8, 0, 0x89, 0xFF][..]);

        let p = VangersCodec.decode(&mut buf).unwrap().unwrap();
        assert_eq!(p.action, Action::SERVER_TIME_QUERY);
        assert_eq!(buf.len(), 1);

        // this is the first byte of a next frame
        assert_eq!(buf[0], 0xFF);
    }
}
