use std::convert::TryFrom;

use ::tokio::io::{AsyncWriteExt, AsyncReadExt};
use ::tokio::net::TcpStream;
use ::tokio::sync::mpsc::{self, Receiver};

use crate::num_traits::FromPrimitive;

use super::protocol::*;

const HS_IN: &'static [u8] = b"Vivat Sicher, Rock'n'Roll forever!!!";
const HS_OUT: &'static [u8] = b"Enter, my son, please...";

pub type ClientID = usize;

pub struct MpscData(pub ClientID, pub Connection);

pub enum Connection {
    Connected,
    // authenticated with protocol version
    Authenticated(u8),
    Disconnected,
    Updated(Packet),
}

impl PartialEq for Connection {
    fn eq(&self, other: &Self) -> bool {
        use Connection::Authenticated as A;
        use Connection::Connected as C;
        use Connection::Disconnected as D;

        match (self, other) {
            (A(_), A(_)) | (C, C) | (D, D) => true,
            _ => false,
        }
    }
}

pub struct Client {
    /// Uniq ClientID
    pub id: ClientID,
    pub connection: Connection,
    pub protocol: u8,
    tx_server: mpsc::Sender<MpscData>,
    tx_client: mpsc::Sender<Vec<u8>>,
}

impl Client {
    pub fn send(&self, packet: &Packet) {
        let tx = self.tx_client.clone();
        let packet = packet.as_bytes();

        ::tokio::spawn(async move {
            if tx.send(packet).await.is_err() {
                println!("Error: send data via mpsc (Server => SendClient)");
            }
        });
    }

    fn event_loop(&self, mut stream: TcpStream, mut rx_server: Receiver<Vec<u8>>) {
        let tx_server = self.tx_server.clone();
        let id = self.id;

        ::tokio::spawn(async move {
            let protocol = match auth(&mut stream).await {
                Ok(protocol_version) => protocol_version,
                Err(err) => {
                    eprintln!("auth failed: {}", err);
                    stream.write(b"Auth failed, bye-bye\0").await.unwrap();
                    stream.shutdown().await.unwrap();
                    if tx_server
                        .send(MpscData(id, Connection::Disconnected))
                        .await
                        .is_err()
                    {
                        println!(
                            "Error: Can't send `Connection::Disconnected` event to server receiver"
                        );
                    }
                    return;
                }
            };

            if tx_server
                .send(MpscData(id, Connection::Authenticated(protocol)))
                .await
                .is_err()
            {
                println!("Error: Can't send `Connection::Auth` event to server receiver");
                return;
            }

            let (mut sr, mut sw) = stream.into_split();

            ::tokio::spawn(async move {
                while let Some(data) = rx_server.recv().await {
                    let event_id = data[2];
                    let q = Action::from_u8(event_id);
                    if q != Some(Action::SERVER_TIME) {
                        // dbg!(("send", id, q.unwrap(), &data[..]));
                    }
                    if sw.write(&data).await.is_err() {
                        println!("error sending data to client");
                        break;
                    }
                }
            });

            let mut buff = [0u8; 32767]; // i16::MAX
            let mut buff_offset: usize = 0;
            loop {
                match sr.read(&mut buff[buff_offset..]).await {
                    Ok(_n @ 0) => {
                        println!("Connection closed by client");
                        tx_server
                            .send(MpscData(id, Connection::Disconnected))
                            .await
                            .ok()
                            .unwrap();
                        break;
                    }
                    Ok(n) => {
                        // let event_size = (buff[0] as u16) as i16 | ((buff[1] as i16) << 8);
                        // if event_size < 0 {
                        //     panic!("Warning: event_size is less than zero");
                        // }
                        // if event_size > i16::try_from(n + 2).unwrap() {
                        //     println!("Warning: event_size is bigger than size of the income data");
                        //     dbg!(event_size, Action::from_u8(buff[2]));
                        // }

                        // total bytes with data: `buff_offset` from previous fetching, `n` - current fetching
                        let buff_readable_size = n + buff_offset;

                        let mut offset = 0;
                        let mut i = 0;
                        while offset < buff_readable_size {
                            i += 1;
                            let packet_size =
                                2 + ((buff[offset] as i16) | ((buff[offset + 1] as i16) << 8));
                            let packet_size = match usize::try_from(packet_size) {
                                Ok(packet_size) => packet_size,
                                Err(_) => {
                                    println!("=================== ERROR ==================");
                                    println!("packet_size < 0, iteration: {}", i);
                                    println!("packet.data.parsed: {:?}", buff[..offset].to_vec());
                                    println!("packet.data.failed: {:?}", buff[offset..].to_vec());
                                    break;
                                }
                            };

                            if packet_size > buff_readable_size - offset {
                                // a tail of the packet will be expect by next reading from the socket
                                // removes all parsed bytes and resets buff_offset
                                buff.copy_within(offset..buff_readable_size, 0);
                                for b in &mut buff[buff_readable_size - offset..] {
                                    *b = 0;
                                }
                                break;
                            }

                            let p = Packet::from_slice(&buff[offset..offset + packet_size]);

                            if tx_server
                                .send(MpscData(id, Connection::Updated(p)))
                                .await
                                .is_err()
                            {
                                panic!("Error: Can't send `Connection::Updated` event to server receiver");
                            }

                            offset += packet_size;
                        }

                        buff_offset = buff_readable_size - offset;
                    }
                    _ => {
                        println!("Connection closed (I/O ERROR)");
                        tx_server
                            .send(MpscData(id, Connection::Disconnected))
                            .await
                            .ok()
                            .unwrap();
                        break;
                    }
                };
            }
        });
    }

    /// Creates new client and sending its to `tx` channel.
    /// Runs separate thread that listening new incoming data.
    pub fn new(stream: TcpStream, tx: mpsc::Sender<MpscData>) -> Self {
        let id = ::rand::random();
        let (tx_client, rx_server) = mpsc::channel::<Vec<u8>>(1000);

        let client = Self {
            protocol: 0,
            id,
            connection: Connection::Connected,
            tx_server: tx,
            tx_client,
        };

        client.event_loop(stream, rx_server);
        client
    }
}

#[derive(thiserror::Error, Debug)]
enum AuthError {
    #[error("Connection closed by client")]
    ClosedByClient,
    #[error("Handshake: unexpected request header")]
    HsUnexpectedRequestHeader,
    #[error("Handshake: unexpected protocol version, expected one of: {0:?}, given: {1}")]
    HsUnexpectedProtocolVersion(&'static [u8], u8),
    #[error("Handshake response fault")]
    HsResponse,
    #[error("Handshake: unexpected request header (zero-terminate symbol is missed)")]
    HsZeroTerminated,
    #[error("Connection fault")]
    Connection,
}

async fn auth(stream: &mut TcpStream) -> Result<u8, AuthError> {
    use AuthError::*;

    let mut buff = [0u8; 256];

    match stream.read(&mut buff).await {
        Ok(_n @ 0) => return Err(ClosedByClient),
        Ok(_n) => {
            if let Some(pos) = buff.iter().position(|&b| b == 0) {
                if !HS_IN.eq(&buff[0..pos]) {
                    return Err(HsUnexpectedRequestHeader);
                }

                let protocol_version = buff[pos + 1];

                if !matches!(protocol_version, 1 | 2) {
                    return Err(HsUnexpectedProtocolVersion(&[1, 2], protocol_version))
                }

                let send = HS_OUT
                    .into_iter()
                    .chain(&[0u8, protocol_version])
                    .map(|&u| u)
                    .collect::<Vec<_>>();

                if let Err(_) = stream.write(&send).await {
                    return Err(HsResponse);
                }

                return Ok(protocol_version);
            } else {
                return Err(HsZeroTerminated)
            }
        }
        _ => return Err(Connection),
    }
}
