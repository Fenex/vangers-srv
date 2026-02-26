use std::sync::Arc;

use ::futures::{SinkExt, StreamExt};
use ::tokio::io::AsyncWriteExt;
use ::tokio::net::{TcpListener, TcpStream};
use ::tokio::sync::mpsc;
use ::tokio_util::codec::Framed;
use ::tower::{Service, ServiceBuilder};
use ::tracing::{error, info};

use crate::ServerConfig;
use crate::client_id::ClientID;
use crate::codec::VangersCodec;
use crate::protocol::Packet;
use crate::server::{SharedState, VangerClient};
use crate::service::{LoggingLayer, VangersHandler, dispatch_responses, handle_disconnect};
use crate::transport::perform_handshake;

pub struct Server {
    pub(in crate::server) conf: ServerConfig,
    pub(in crate::server) state: Arc<SharedState>,
}

impl Server {
    pub fn new(conf: ServerConfig) -> Self {
        Self {
            conf,
            state: Arc::new(SharedState::new()),
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let endpoint = format!("0.0.0.0:{}", self.conf.port);
        info!("Server listening on {}", endpoint);
        let listener = TcpListener::bind(&endpoint).await?;

        loop {
            let (stream, addr) = match listener.accept().await {
                Ok(ok) => ok,
                Err(e) => {
                    error!("accept error: {}", e);
                    continue;
                }
            };
            info!("new client from {}", addr);
            let state = Arc::clone(&self.state);
            tokio::spawn(async move {
                if let Err(e) = serve_connection(state, stream, addr).await {
                    error!("connection error: {}", e);
                }
            });
        }
    }
}

async fn serve_connection(
    state: Arc<SharedState>,
    stream: TcpStream,
    addr: std::net::SocketAddr,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut stream = stream;
    let protocol = match perform_handshake(&mut stream).await {
        Ok(p) => p,
        Err(e) => {
            let _ = stream.shutdown().await;
            Err(e)?
        }
    };

    let framed = Framed::new(stream, VangersCodec);
    let (mut write_sink, read_stream) = framed.split();

    let client_id: ClientID = rand::random();
    let (tx, mut rx) = mpsc::channel::<Packet>(1000);
    {
        let client = VangerClient {
            id: client_id,
            ip: addr,
            protocol,
            tx,
        };
        state.clients.write().await.insert(client_id, client);
    }

    ::tokio::spawn(async move {
        while let Some(packet) = rx.recv().await {
            if write_sink.send(packet).await.is_err() {
                break;
            }
        }
    });

    let mut svc = ServiceBuilder::new()
        .layer(LoggingLayer::new())
        .service(VangersHandler::new(state.clone()));

    let mut read_stream = read_stream.map(|r| r.map_err(|e| e.to_string()));
    while let Some(result) = read_stream.next().await {
        let packet = match result {
            Ok(p) => p,
            Err(e) => {
                error!("decode error: {}", e);
                break;
            }
        };
        match svc.call((client_id, packet)).await {
            Ok(actions) => dispatch_responses(&state, client_id, actions).await,
            Err(e) => {
                error!("handler error: {}", e);
            }
        }
    }

    state.clients.write().await.remove(&client_id);
    handle_disconnect(&state, client_id).await;
    Ok(())
}
