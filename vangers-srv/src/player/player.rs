use std::cell::RefCell;
use std::rc::Rc;

use ::log::info;

use crate::game::World;
use crate::protocol::NetTransportReceive;
use crate::{client::ClientID, vanject::Pos};

use super::Auth;
use super::Bind;
use super::Body;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Status {
    INITIAL = 0,
    GAMING = 1,
    FINISHED = 2,
}

#[derive(Debug)]
pub struct Player {
    /// The id that bind `Player` & `Client` structs.
    pub client_id: ClientID,
    /// Storage data to sync with client-side ID.
    pub bind: Option<Bind>,
    /// Storage name, password and other related functionality.
    pub auth: Option<Auth>,
    /// Storage several info that sending via TCP
    pub body: Option<Body>,
    /// Player is assign to this world
    pub world: Option<Rc<RefCell<World>>>,

    pub pos: Pos<i16>,

    pub status: Status,
}

impl Player {
    pub fn new(client_id: ClientID) -> Self {
        Self {
            client_id,
            bind: None,
            auth: None,
            body: None,
            world: None,
            pos: Pos::default(),
            status: Status::INITIAL,
        }
    }

    // pub fn get_inventory_vanject_ids(&self) -> Vec<i32> {
    //     match (&self.world, &self.bind) {
    //         (Some(world), Some(bind)) => world
    //             .as_ref()
    //             .borrow()
    //             .vanjects
    //             .iter()
    //             .filter_map(|(i, v)| {
    //                 if v.get_station() == bind.id as i32 {
    //                     Some(i)
    //                 } else {
    //                     None
    //                 }
    //             })
    //             .map(|&b| b)
    //             .collect::<Vec<_>>(),
    //         _ => vec![],
    //     }
    // }

    #[allow(dead_code)]
    pub fn bind_reset(&mut self) {
        self.set_bind(0);
    }

    pub fn set_bind(&mut self, id: u8) {
        if id == 0 {
            self.bind = None;
            self.auth = None;
        } else {
            self.bind = Some(Bind::new(id));
        }
    }

    pub fn set_auth(&mut self, name: &[u8], pwd: &[u8]) {
        #[inline(always)]
        fn check_for_null_terminate(s: &[u8]) -> bool {
            s.len() > 1 && s[s.len() - 1] == 0
        }

        if !check_for_null_terminate(name) {
            panic!("player::set_auth: expected null-terminated cstring");
        }

        self.auth = Some(Auth::new(name, pwd));
    }

    pub fn set_body(&mut self, slice: &[u8]) -> Result<(), &'static str> {
        self.body = Body::from_slice(slice);

        match self.body {
            Some(ref body) => {
                info!(
                    "set body with color {} for player {}",
                    body.color, self.client_id
                );
                Ok(())
            }
            None => Err("parse slice as PlayerBody failed"),
        }
    }
}
