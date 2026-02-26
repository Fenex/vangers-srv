//! Shared server state for Tower-based connection handling.
//! Used by `VangersHandler` and per-connection tasks.

use std::sync::atomic::{AtomicU32, Ordering};

use ::tokio::sync::{RwLock, mpsc};

use crate::client_id::ClientID;
use crate::protocol::Packet;

use super::games::Games;
use crate::utils::Uptime;

/// Registry of connected clients: maps client ID to the channel used to send packets to that client.
pub type ClientRegistry = std::collections::HashMap<ClientID, mpsc::Sender<Packet>>;

/// Protocol version per client (set after handshake in per-connection task).
pub type ClientProtocolMap = std::collections::HashMap<ClientID, u8>;

/// Shared state accessible from all connection tasks and the handler service.
pub struct SharedState {
    pub games: RwLock<Games>,
    pub clients: RwLock<ClientRegistry>,
    /// Protocol version (1 or 2) per client; used e.g. to send Z_TIME_RESPONSE only for protocol > 1.
    pub clients_protocol: RwLock<ClientProtocolMap>,
    pub uptime: Uptime,
    games_id_counter: AtomicU32,
}

impl SharedState {
    pub fn new() -> Self {
        Self {
            games: RwLock::new(Games::new()),
            clients: RwLock::new(ClientRegistry::new()),
            clients_protocol: RwLock::new(ClientProtocolMap::new()),
            uptime: Uptime::new(),
            games_id_counter: AtomicU32::new(1),
        }
    }

    /// Allocate a new unique game ID (1, 2, 3, ...). Matches old Server::get_game_uniq_id behavior.
    pub fn next_game_id(&self) -> u32 {
        self.games_id_counter.fetch_add(1, Ordering::SeqCst)
    }

    /// Server uptime in seconds (for SERVER_TIME_QUERY response: uptime * 256).
    pub fn uptime_secs(&self) -> u32 {
        self.uptime.as_secs_u32()
    }
}

impl Default for SharedState {
    fn default() -> Self {
        Self::new()
    }
}
