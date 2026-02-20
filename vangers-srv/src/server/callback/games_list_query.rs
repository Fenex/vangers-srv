use std::ffi::CString;

use super::{OnUpdateError, OnUpdateOk};
use crate::Server;
use crate::client::ClientID;
use crate::game::Type as GameType;
use crate::protocol::Packet;

#[allow(non_camel_case_types)]
pub(super) trait OnUpdate_GamesListQuery {
    fn games_list_query(
        &mut self,
        packet: &Packet,
        client_id: ClientID,
    ) -> Result<OnUpdateOk, OnUpdateError>;
}

impl OnUpdate_GamesListQuery for Server {
    #[tracing::instrument(skip_all)]
    fn games_list_query(
        &mut self,
        packet: &Packet,
        _client_id: ClientID,
    ) -> Result<OnUpdateOk, OnUpdateError> {
        let mut games_count: u8 = 0;
        let mut data = vec![games_count];
        for game in self.games.values() {
            if !game.is_configured() {
                continue;
            }

            let str_gmtype = match game.get_gmtype() {
                GameType::MECHOSOMA => 'M',
                GameType::VAN_WAR => 'V',
                GameType::PASSEMBLOSS => 'P',
                GameType::MIR_RAGE => 'R',
                _ => '?',
            };

            let title_head = String::from("[Rust-SRV] ");
            let title_tail = format!(
                ": {} {} {}",
                game.players.len(),
                str_gmtype,
                game.birth_time
            );

            let title = match game.name.last() {
                Some(0) => &game.name[..game.name.len() - 1],
                Some(_) => &game.name[..],
                None => b"[UNDEFINED TITLE]",
            };

            let mut name = std::iter::empty()
                .chain(CString::new(title_head).unwrap().as_bytes())
                .chain(title)
                .chain(CString::new(title_tail).unwrap().as_bytes())
                .chain(&[0]) // null terminator of the c-style string
                .copied()
                .collect::<Vec<_>>();

            data.append(&mut game.id.to_le_bytes().to_vec());
            data.append(&mut name);

            games_count += 1;
        }

        data[0] = games_count;

        packet
            .create_answer(data)
            .map(OnUpdateOk::Response)
            .ok_or(OnUpdateError::ResponsePacketTypeNotExist(packet.action))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::game::Config;
    use crate::game::Game;
    use crate::player::{Body as PlayerBody, Player};
    use crate::protocol::Action;

    trait MockServer {
        fn create_without_games() -> Server {
            Server::new(Default::default())
        }

        /// Returns Server with one (U)nconfigured (G)ame:
        #[allow(non_snake_case)]
        fn create_UG() -> Server {
            let mut srv = Server::new(Default::default());
            let mut game = Game::new(1);
            game.attach_player(Player::new(11));
            srv.games.insert(1, game);
            srv
        }

        /// Returns Server with one (C)onfigured (G)ames and one player:
        #[allow(non_snake_case)]
        fn create_CG_P1() -> Server {
            let mut srv = Server::new(Default::default());

            let mut game = Game::new(1);
            game.config = Some(Config::new(GameType::PASSEMBLOSS));

            let mut player = Player::new(11);
            player.set_auth(b"player\0", b"\0");
            player.body = Some(PlayerBody::default());

            game.attach_player(player);
            srv.games.insert(1, game);
            srv
        }

        /// Returns Server with two games:
        ///  - (U)nconfigured (G)ame with one player
        ///  - (C)onfigured (G)ame with two players
        #[allow(non_snake_case)]
        fn create_UG_P1_CG_P2() -> Server {
            let mut srv = Server::new(Default::default());

            {
                let mut game = Game::new(1);
                game.attach_player(Player::new(11));
                srv.games.insert(1, game);
            }

            {
                let mut game = Game::new(2);
                game.config = Some(Config::new(GameType::PASSEMBLOSS));

                let mut player = Player::new(21);
                player.set_auth(b"player1\0", b"\0");
                player.body = Some(PlayerBody::default());
                game.attach_player(player);

                let mut player = Player::new(22);
                player.set_auth(b"player2\0", b"\0");
                player.body = Some(PlayerBody::default());
                game.attach_player(player);

                srv.games.insert(2, game);
            }

            srv
        }
    }

    impl MockServer for Server {}

    fn get_request_packet() -> Packet {
        Packet::new(Action::GAMES_LIST_QUERY, &[])
    }

    #[test]
    fn empty_games() {
        let request = get_request_packet();

        let mut srv = Server::create_without_games();
        let query_response = srv.games_list_query(&request, 1);
        assert!(query_response.is_ok());

        match query_response.unwrap() {
            OnUpdateOk::Response(p) => {
                assert_eq!(p.action, Action::GAMES_LIST_RESPONSE);
                assert_eq!(p.data, &[0]);
            }
            t => panic!("unexpected responsed type: {:?}", t),
        }
    }

    #[test]
    fn unconfigured_game() {
        let request = get_request_packet();
        let mut srv = Server::create_UG();
        let query_response = srv.games_list_query(&request, 1);
        assert!(query_response.is_ok());

        match query_response.unwrap() {
            OnUpdateOk::Response(p) => {
                assert_eq!(p.action, Action::GAMES_LIST_RESPONSE);
                assert_eq!(p.data, &[0]);
            }
            t => panic!("unexpected responsed type: {:?}", t),
        }
    }

    #[test]
    fn configured_games_1_players_1() {
        let request = get_request_packet();
        let mut srv = Server::create_CG_P1();
        let query_response = srv.games_list_query(&request, 1);
        assert!(query_response.is_ok());

        match query_response.unwrap() {
            OnUpdateOk::Response(p) => {
                assert_eq!(p.action, Action::GAMES_LIST_RESPONSE);
                assert_eq!(p.data[0], 1); // count of games
                assert_eq!(p.data[1..5], 1u32.to_le_bytes()); // id of the single game
                assert_eq!(
                    &p.data[5..],
                    b"[Rust-SRV] [UNDEFINED TITLE]: 1 P 0:00:00\0".to_vec()
                );
            }
            t => panic!("unexpected responsed type: {:?}", t),
        }
    }

    #[test]
    fn configured_games_2_players_1_2() {
        let request = get_request_packet();
        let mut srv = Server::create_UG_P1_CG_P2();
        let query_response = srv.games_list_query(&request, 1);
        assert!(query_response.is_ok());

        match query_response.unwrap() {
            OnUpdateOk::Response(p) => {
                assert_eq!(p.action, Action::GAMES_LIST_RESPONSE);
                assert_eq!(p.data[0], 1); // count of games
                assert_eq!(p.data[1..5], 2u32.to_le_bytes()); // id of the single game
                assert_eq!(
                    &p.data[5..],
                    b"[Rust-SRV] [UNDEFINED TITLE]: 2 P 0:00:00\0".to_vec()
                );
            }
            t => panic!("unexpected responsed type: {:?}", t),
        }
    }
}
