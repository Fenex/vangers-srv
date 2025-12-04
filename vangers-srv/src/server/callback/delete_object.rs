use ::log::debug;

use crate::client::ClientID;
use crate::protocol::{Action, Packet};
use crate::utils::slice_le_to_i32;
use crate::Server;

use super::{OnUpdateError, OnUpdateOk};

#[derive(Debug, ::thiserror::Error)]
pub enum DeleteObjectError {
    // #[error("fail read slice as vanject")]
    // SliceToVanjectParse,
    #[error("player with `client_id`={0} not found")]
    PlayerNotFound(ClientID),
    #[error("player with `client_id`={0} not bind")]
    PlayerNotBind(ClientID),
}

#[allow(non_camel_case_types)]
pub(super) trait OnUpdate_DeleteObject {
    fn delete_object(
        &mut self,
        packet: &Packet,
        client_id: ClientID,
    ) -> Result<OnUpdateOk, OnUpdateError>;
}

impl OnUpdate_DeleteObject for Server {
    fn delete_object(
        &mut self,
        packet: &Packet,
        client_id: ClientID,
    ) -> Result<OnUpdateOk, OnUpdateError> {
        let vanject_id = slice_le_to_i32(&packet.data[0..4]);

        let game = match self.get_mut_game_by_clientid(client_id) {
            Some(game) => game,
            None => return Err(DeleteObjectError::PlayerNotFound(client_id).into()),
        };

        let player_auth_id = {
            match game.get_mut_player(client_id).unwrap().bind {
                Some(bind) => bind.id(),
                None => return Err(DeleteObjectError::PlayerNotBind(client_id).into()),
            }
        };

        let data = std::iter::empty()
            .chain(&vanject_id.to_le_bytes())
            .chain(&[player_auth_id])
            .chain(&packet.data[4..8])
            .chain(&packet.data[8..])
            .copied()
            .collect::<Vec<_>>();

        let answer = Packet::new(Action::DELETE_OBJECT, &data);

        // match game.vanjects.remove(&vanject_id) {
        //     Some(v) => {
        //         if v.is_private() {
        //             println!(
        //                 "DELETE OBJECT: deleted PRIVATE vanject: {:?}",
        //                 v.id.to_le_bytes()
        //             );
        //         }
        //     }
        //     None => println!(
        //         "DELETE OBJECT: VANJECT with id=`{:?}` not found",
        //         &vanject_id.to_le_bytes()
        //     ),
        // }

        if game.vanjects.remove(&vanject_id).is_none() {
            debug!("VANJECT with id=`{}` not found", vanject_id);
        }

        self.notify_game(client_id, &answer);
        Ok(OnUpdateOk::Complete)
    }
}
