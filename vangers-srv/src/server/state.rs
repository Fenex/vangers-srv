//! Shared server state for Tower-based connection handling.
//! Used by `VangersHandler` and per-connection tasks.

use std::net::SocketAddr;
use std::sync::atomic::{AtomicU32, Ordering};

use ::tokio::sync::{RwLock, mpsc};

use crate::client_id::ClientID;
use crate::protocol::Packet;

use super::games::Games;
use crate::utils::Uptime;

/// Подключённый клиент: адрес, версия протокола и канал для отправки пакетов.
#[derive(Debug)]
pub struct VangerClient {
    pub id: ClientID,
    pub ip: SocketAddr,
    pub protocol: u8,
    pub(crate) tx: mpsc::Sender<Packet>,
}

/// Реестр подключённых клиентов.
pub type ClientRegistry = std::collections::HashMap<ClientID, VangerClient>;

/// Shared state accessible from all connection tasks and the handler service.
pub struct SharedState {
    pub games: RwLock<Games>,
    pub clients: RwLock<ClientRegistry>,
    pub uptime: Uptime,
    games_id_counter: AtomicU32,
}

impl SharedState {
    pub fn new() -> Self {
        Self {
            games: RwLock::new(Games::new()),
            clients: RwLock::new(ClientRegistry::new()),
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
