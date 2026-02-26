//! Dispatches `ResponseAction`s to clients via `ClientRegistry`.

use crate::client_id::ClientID;
use crate::server::SharedState;
use crate::server::games::Games;

use super::Target;

fn resolve_target_ids_sync(sender_id: ClientID, target: &Target, games: &Games) -> Vec<ClientID> {
    match target {
        Target::Sender => vec![sender_id],
        Target::Specific(ids) => ids.clone(),
        Target::GameExceptSender | Target::AllInGame => {
            let game = games
                .iter()
                .find(|(_, g)| g.get_player(sender_id).is_some());
            match game {
                Some((_, game)) => {
                    let include_sender = matches!(target, Target::AllInGame);
                    game.players
                        .iter()
                        .filter(|p| p.bind.is_some())
                        .map(|p| p.client_id)
                        .filter(|&id| include_sender || id != sender_id)
                        .collect()
                }
                None => vec![],
            }
        }
    }
}

/// Send `actions` to the appropriate clients using `state.clients`.
/// Called from the per-connection task after each `VangersHandler::call`.
pub async fn dispatch_responses(
    state: &SharedState,
    sender_id: ClientID,
    actions: Vec<super::ResponseAction>,
) {
    let targets = {
        let games = state.games.read().await;
        actions
            .iter()
            .map(|a| resolve_target_ids_sync(sender_id, &a.target, &games))
            .collect::<Vec<_>>()
    };

    let clients = state.clients.read().await;
    for (action, ids) in actions.into_iter().zip(targets) {
        for id in ids {
            if let Some(tx) = clients.get(&id) {
                let _ = tx.send(action.packet.clone()).await;
            }
        }
    }
}
