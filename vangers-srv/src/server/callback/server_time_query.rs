use crate::client::ClientID;
use crate::protocol::Packet;
use crate::Server;

use super::{OnUpdateError, OnUpdateOk};

#[allow(non_camel_case_types)]
pub(super) trait OnUpdate_ServerTimeQuery {
    fn server_time_query(
        &mut self,
        packet: &Packet,
        client_id: ClientID,
    ) -> Result<OnUpdateOk, OnUpdateError>;
}

impl OnUpdate_ServerTimeQuery for Server {
    fn server_time_query(
        &mut self,
        packet: &Packet,
        _client_id: ClientID,
    ) -> Result<OnUpdateOk, OnUpdateError> {
        let data = (self.uptime() * 256).to_le_bytes();

        packet
            .create_answer(data.to_vec())
            .map(OnUpdateOk::Response)
            .ok_or(OnUpdateError::ResponsePacketTypeNotExist(packet.action))
    }
}
