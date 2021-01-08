use std::thread::sleep;
use std::time::Duration;

use ::tokio::net::TcpListener;
use ::tokio::sync::mpsc;

use crate::client::{Client, ClientID, Connection, MpscData};
use crate::game::Game;
use crate::protocol::*;
use crate::server::callback::*;
use crate::utils::Uptime;

use super::games::Games;

enum Event {
    Add(Client),
    #[allow(dead_code)]
    Halt,
}

pub struct Server {
    /// The port that is listening for attempting new TCP connections.
    port: u16,
    /// List of all games on the server.
    pub(in crate::server) games: Games,
    /// Counter that storages an uniq game_id for next new game.
    /// TODO: Replace to iterator (i++) (?)
    games_id_uniq: u32,
    /// List of all connected TCP clients.
    pub(in crate::server) clients: Vec<Client>,
    /// Uptime server
    uptime: Uptime,
    // get_game_uniq_id: Box<dyn Fn() -> i32>
}

impl Server {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            games: Games::new(),
            games_id_uniq: 0,
            clients: vec![],
            uptime: Uptime::new(),
            // shell: None,
            // get_game_uniq_id: Box::new(q),
        }
    }

    // pub fn enable_shell(&mut self) {
    //     self.shell = Some(::tokio::spawn(async move {
    //         println!("Interactive shell enabled");
    //         let stdin = io::stdin();
    //         let stdout = io::stdout();

    //         loop {
    //             {
    //                 let mut stdout = stdout.lock();
    //                 stdout.write_all("vangers-srv shell> ".as_bytes()).unwrap();
    //                 stdout.flush().unwrap();
    //             }

    //             let mut input = String::new();
    //             match stdin.read_line(&mut input) {
    //                 Ok(_) => match input.trim() {
    //                     "" => continue,
    //                     "quit" | "exit" => {
    //                         println!("Interactive shell will be terminated");
    //                         return;
    //                     }
    //                     cmd => {
    //                         let cmd = std::iter::once("").chain(cmd.split_whitespace());

    //                         match ShellCmd::try_parse_from(cmd) {
    //                             Ok(shell) => println!("OK command: {:?}", shell),
    //                             Err(err) => println!("Error command: {}", err),
    //                         }
    //                     }
    //                 },
    //                 Err(error) => println!("error: {}", error),
    //             }
    //         }
    //     }));
    // }

    /// Returns the time since server was started in milliseconds.
    pub fn uptime(&self) -> u32 {
        self.uptime.as_secs_u32()

        // // sef.start_time is Instant;
        // let uptime = self.start_time.elapsed().as_millis();
        // u32::try_from(uptime).unwrap_or_else(|_| {
        //     // reset uptime to zero if u32 overflow has been
        //     // detected (~49 days)
        //     // the idia is the same as `SDL_GetTicks` method
        //     self.start_time = Instant::now();
        //     0
        // })
    }

    pub(in crate::server) fn get_game_by_clientid(&self, client_id: ClientID) -> Option<&Game> {
        self.games.get_game_by_client_id(client_id)
    }

    pub(in crate::server) fn get_mut_game_by_clientid(
        &mut self,
        client_id: ClientID,
    ) -> Option<&mut Game> {
        self.games.get_mut_game_by_client_id(client_id)
    }

    pub(in crate::server) fn get_game_uniq_id(&mut self) -> u32 {
        self.games_id_uniq += 1;
        self.games_id_uniq
    }

    pub fn notify(
        &self,
        client_id: ClientID,
        packet: &Packet,
        filter: Box<dyn Fn(&ClientID) -> bool>,
    ) {
        let game = match self.get_game_by_clientid(client_id) {
            Some(game) => game,
            None => {
                panic!(
                    "Player with client_id={} not found on the server",
                    client_id
                );
            }
        };

        let client_ids = game
            .players
            .iter()
            .map(|p| p.client_id)
            .filter(filter)
            .collect::<Vec<_>>();

        self.clients
            .iter()
            .filter(|c| client_ids.iter().any(|&id| id == c.id))
            .for_each(|c| c.send(&packet));
    }

    /// Sends `packet` to the current client only.
    pub fn notify_player(&self, client_id: ClientID, packet: &Packet) {
        self.notify(client_id, packet, Box::new(move |&id| id == client_id));
    }

    /// Sends `packet` to all clients in the game exclude a caller client `client_id`.
    pub fn notify_game(&self, client_id: ClientID, packet: &Packet) {
        self.notify(client_id, packet, Box::new(move |&id| id != client_id));
    }

    /// Sends `packet` to all clients.
    pub fn notify_all(&self, client_id: ClientID, packet: &Packet) {
        self.notify(client_id, packet, Box::new(|_| true));
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (client_tx, mut clients_rx) = mpsc::channel(50);
        let (event_tx, mut event_rx) = mpsc::channel::<Event>(10);

        let endpoint = format!("0.0.0.0:{}", self.port);
        let listener = TcpListener::bind(endpoint).await?;

        ::tokio::spawn(async move {
            // listening for connecting new clients
            loop {
                if let Ok((stream, _)) = listener.accept().await {
                    println!("====== new client connected ======");
                    let client = Client::new(stream, client_tx.clone());
                    if event_tx.send(Event::Add(client)).await.is_err() {
                        println!("Terminate tcp-listener because of `event_rx` was closed.");
                        break;
                    }
                }
            }
        });

        let sleep_duration = Duration::from_millis(10);
        loop {
            // match event_rx_shell.try_recv() {
            //     Ok(ShellCmd)
            // };

            match event_rx.try_recv() {
                Ok(Event::Add(client)) => {
                    self.clients.push(client);
                    continue;
                }
                Ok(Event::Halt) => {
                    return Ok(());
                }
                // Ok(Event::ShellCmd(cmd)) => self.do_shell(cmd),
                _ => {}
            };

            if let Ok(data) = clients_rx.try_recv() {
                match data {
                    MpscData(id, Connection::Disconnected) => {
                        self.close_socket(&Packet::new(Action::CLOSE_SOCKET, &[]), id)
                            .ok();
                        // if let Some(game) = self.get_mut_game_by_clientid(id) {
                        //     game.players.retain(|p| p.client_id != id)
                        // }
                        self.clients.retain(|c| c.id != id);
                        // if let Some(client) = self.clients.iter_mut().find(|c| c.id == id) {
                        //     client.connection = Connection::Disconnected;
                        // }
                        // self.clients
                        //     .retain(|c| c.connection != Connection::Disconnected);
                    }
                    MpscData(id, connection @ Connection::Authenticated)
                    | MpscData(id, connection @ Connection::Connected) => {
                        let client = self.clients.iter_mut().find(|c| c.id == id);
                        if let Some(client) = client {
                            client.connection = connection;
                        }
                    }
                    MpscData(id, Connection::Updated(p)) => {
                        self.on_update(id, p);
                    }
                };

                continue;
            }

            sleep(sleep_duration);
        }
    }
}
