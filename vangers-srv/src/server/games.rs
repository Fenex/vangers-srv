use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use crate::{client::ClientID, player::Player};

use crate::game::*;

pub struct Games(HashMap<GameID, Game>);

impl Games {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    #[allow(dead_code)]
    pub fn get_game_by_id(&self, id: GameID) -> Option<&Game> {
        self.get(&id)
    }

    pub fn get_mut_game_by_id(&mut self, id: GameID) -> Option<&mut Game> {
        self.get_mut(&id)
    }

    pub fn get_game_by_client_id(&self, client_id: ClientID) -> Option<&Game> {
        self.iter()
            .find(|(_, game)| game.get_player(client_id).is_some())
            .map(|(_, game)| game)
    }

    pub fn get_mut_game_by_client_id(&mut self, client_id: ClientID) -> Option<&mut Game> {
        self.iter_mut()
            .find(|(_, game)| game.get_player(client_id).is_some())
            .map(|(_, game)| game)
    }

    #[allow(dead_code)]
    pub fn get_player_by_client_id(&self, client_id: ClientID) -> Option<(&Game, &Player)> {
        self.get_game_by_client_id(client_id)
            .and_then(|game| game.get_player(client_id).map(|player| (game, player)))
    }

    pub fn get_mut_player_by_client_id(&mut self, client_id: ClientID) -> Option<&mut Player> {
        self.get_mut_game_by_client_id(client_id)
            .and_then(|game| game.get_mut_player(client_id))
    }

    // TODO: convert `Err(String)` into `Err(Enum\Struct)`
    pub fn create(&mut self, id: GameID) -> Result<GameID, String> {
        if self.contains_key(&id) {
            return Err(format!("game already exists with id=`{}`", id));
        }

        self.insert(id, Game::new(id));

        Ok(id)
    }
}

impl Deref for Games {
    type Target = HashMap<GameID, Game>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Games {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
