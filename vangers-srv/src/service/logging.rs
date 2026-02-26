//! Logging middleware for the Tower pipeline.

use std::task::{Context, Poll};

use tower::{Layer, Service};
use tracing::info;

use crate::client_id::ClientID;
use crate::protocol::Packet;

/// Layer that wraps a service and logs each request (client_id, packet action).
#[derive(Debug, Clone, Default)]
pub struct LoggingLayer;

impl LoggingLayer {
    pub fn new() -> Self {
        Self
    }
}

impl<S> Layer<S> for LoggingLayer {
    type Service = LoggingService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LoggingService { inner }
    }
}

/// Service that logs `(ClientID, Packet)` then delegates to the inner service.
pub struct LoggingService<S> {
    inner: S,
}

impl<S> Service<(ClientID, Packet)> for LoggingService<S>
where
    S: Service<(ClientID, Packet)>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, (client_id, packet): (ClientID, Packet)) -> Self::Future {
        info!("[<-] client_id={} action={:?}", client_id, packet.action);
        self.inner.call((client_id, packet))
    }
}
