use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::CStr;
use std::rc::Rc;

use crate::client::ClientID;
use crate::player::{Player, Status as PlayerStatus};
use crate::protocol::NetTransportReceive;
use crate::utils::Uptime;
use crate::vanject::Vanject;

use super::Config;
use super::Type;
use super::World;

const MAX_PLAYER_ID: u8 = 30u8; // or 31 (?)
const MIN_PLAYER_ID: u8 = 1u8;

pub type GameID = u32;

pub struct Game {
    pub id: GameID,
    pub name: Vec<u8>,
    pub players: Vec<Player>,
    pub worlds: Vec<Rc<RefCell<World>>>,
    pub birth_time: Uptime,
    pub config: Option<Config>,
    pub vanjects: HashMap<i32, Vanject>,
    // Bitwise field. Used to marks which players' ids already taken.
    // Each bit marks its own player.
    // used_players_ids: u32
}

impl std::fmt::Debug for Game {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        let name = match CStr::from_bytes_with_nul(&self.name) {
            Ok(cstr) => cstr.to_string_lossy(),
            Err(_) => Cow::Borrowed("[CORRUPT NAME]"),
        };

        fmt.debug_struct("Game")
            .field("id", &self.id)
            .field("name", &name)
            .field("birth_time", &self.birth_time)
            .field("players", &self.players)
            .field("worlds", &self.worlds)
            .field("config", &self.config)
            .field("vanjects_count", &self.vanjects.len())
            .finish()
    }
}

impl Game {
    pub fn is_configured(&self) -> bool {
        self.config.is_some()
    }

    pub fn new(id: u32) -> Self {
        Self {
            id,
            name: vec![],
            players: vec![],
            worlds: vec![],
            birth_time: Uptime::new(),
            config: None, // used_players_ids: 0,
            vanjects: HashMap::new(),
        }
    }

    pub fn get_player(&self, client_id: ClientID) -> Option<&Player> {
        self.players.iter().find(|p| p.client_id == client_id)
    }

    pub fn get_mut_player(&mut self, client_id: ClientID) -> Option<&mut Player> {
        self.players.iter_mut().find(|p| p.client_id == client_id)
    }

    /// Returns new unique `player_id` if the game has free player slots.
    fn get_uniq_player_id(&self) -> Option<u8> {
        let mut ids = self
            .players
            .iter()
            .filter(|&p| p.bind.is_some())
            .map(|p| p.bind.unwrap().id())
            .collect::<Vec<_>>();
        ids.sort();

        let mut iter = ids.iter();

        for i in MIN_PLAYER_ID..=MAX_PLAYER_ID {
            match iter.next() {
                Some(&id) if id == i => continue,
                _ => return Some(i),
            }
        }

        None
    }

    // pub fn add_vanject(
    //     &mut self,
    //     vanject: Vanject,
    //     client_id: ClientID,
    // ) -> Result<Option<crate::protocol::Packet>, &'static str> {
    //     let player = match self.players.iter_mut().find(|p| p.client_id == client_id) {
    //         Some(p) => p,
    //         None => return Err("player not found"),
    //     };

    //     if vanject.is_non_global() && player.world.is_none() {
    //         return Err("create vanject before set world");
    //     }

    //     let player_bind_id = match &player.bind {
    //         Some(bind) => bind.id,
    //         None => return Err("player not bind"),
    //     };

    //     if vanject.get_type() == NID::VANGER {
    //         let mut world = player.world.as_ref().unwrap().borrow_mut();
    //         player.pos_x = vanject.pos.x;
    //         player.pos_y = vanject.pos.y;
    //         world.vanjects.insert(vanject.id, vanject);

    //         let data = std::iter::empty()
    //             .chain(&[player_bind_id])
    //             .chain(&player.pos_x.to_le_bytes())
    //             .chain(&player.pos_y.to_le_bytes())
    //             .map(|&b| b)
    //             .collect::<Vec<_>>();

    //         let answer =
    //             crate::protocol::Packet::new(crate::protocol::Action::PLAYERS_POSITION, &data);
    //         // self.notify_game(&answer, client_id);
    //         return Ok(Some(answer));
    //     //Original-C: world.process_set_position(player);
    //     } else {
    //         // println!("ignore non-vanger vanject");
    //     }

    //     return Ok(None);

    //     // if vanject.is_non_global() {
    //     //     let mut world = player.world.as_ref().unwrap().borrow_mut();
    //     //     if world.vanjects.contains_key(&vanject.id) {
    //     //         println!(
    //     //             "CREATE_OBJECT: vanject with that id is already exists in the WORLD, ignore"
    //     //         );
    //     //         // todo!()
    //     //     }

    //     //     if vanject.get_type() == NID::VANGER {
    //     //         player.pos_x = vanject.pos.x;
    //     //         player.pos_y = vanject.pos.y;
    //     //         world.vanjects.insert(vanject.id, vanject);
    //     //     //Original-C: world.process_set_position(player);
    //     //     } else if !vanject.is_players() {
    //     //         world.vanjects.insert(vanject.id, vanject);
    //     //     } else {
    //     //         world.vanjects.insert(vanject.id, vanject);
    //     //         //Original-C:  world -> process_create_inventory(this, obj);
    //     //     }
    //     // } else {
    //     //     if self.vanjects.contains_key(&vanject.id) {
    //     //         println!(
    //     //             "CREATE_OBJECT: vanject with id=`{}` is already exists in the GAME, ignore",
    //     //             vanject.id
    //     //         );
    //     //         // todo!()
    //     //     }
    //     //     self.vanjects.insert(vanject.id, vanject);
    //     //     //Original-C:  game -> process_create_globals(!(in_buffer.current_event() & ECHO_EVENT) ? this : 0, obj);
    //     // }

    //     // Ok(())
    // }

    /// Assign player to the world.
    /// Returns `true` if player status has been changed
    pub fn place_player(&mut self, client_id: ClientID, world: &World) -> bool {
        if let (Some(p), Some(w)) = (
            self.players.iter_mut().find(|p| p.client_id == client_id),
            self.worlds.iter().find(|w| w.borrow().id == world.id),
        ) {
            p.world = Some(Rc::clone(w));
            if p.status != PlayerStatus::GAMING {
                p.status = PlayerStatus::GAMING;
                true
            } else {
                false
            }
        } else {
            panic!("given player or\\and world are not includes in internal arrays");
        }
    }

    /// Try to attach `p` player to the game.
    /// Returns attached player's id if player was attached sucessfully, otherwise `0`.
    pub fn attach_player(&mut self, mut p: Player) -> Option<u8> {
        match self.get_uniq_player_id() {
            Some(uniq_id) if uniq_id > 0 => {
                p.set_bind(uniq_id);
                self.players.push(p);
                Some(uniq_id)
            }
            _ => None,
        }
    }

    pub fn set_config(&mut self, slice: &[u8]) -> Result<(), &'static str> {
        if self.is_configured() {
            return Err("already configured");
        }

        self.config = Config::from_slice(slice);
        match self.config {
            Some(_) => Ok(()),
            None => Err("fail converting slice to config"),
        }
    }

    #[allow(dead_code)]
    pub fn get_gmtype(&self) -> Type {
        match self.config {
            Some(ref a) => a.get_gametype(),
            _ => Type::UNCONFIGURED,
        }
    }
}
