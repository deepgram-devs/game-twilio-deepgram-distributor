use crate::message::Message;
use crate::state::State;
use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    Extension,
};
use futures::stream::StreamExt;
use futures::SinkExt;
use std::sync::Arc;

pub async fn game_handler(
    ws: WebSocketUpgrade,
    Extension(state): Extension<Arc<State>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn pop_a_game_code(state: Arc<State>) -> Option<String> {
    let mut game_codes = state.game_codes.lock().await;
    let game_code = game_codes
        .iter()
        .map(|x| x.clone())
        .collect::<Vec<String>>()
        .pop();

    if let Some(game_code) = game_code.clone() {
        game_codes.remove(&game_code);
    }

    game_code
}

async fn handle_socket(socket: WebSocket, state: Arc<State>) {
    let (mut game_sender, mut game_reader) = socket.split();

    let game_code = pop_a_game_code(state.clone()).await;

    match game_code {
        Some(game_code) => {
            // we add this manual scoping so that we drop the games lock after this logic
            {
                let mut games = state.games.lock().await;

                // tell the game the phone number to call
                game_sender
                    .send(Message::Text(state.twilio_phone_number.clone()).into())
                    .await
                    .expect("Failed to send the phone number to the game.");

                // tell the game what game code we are assigning it
                game_sender
                    .send(Message::Text(game_code.clone()).into())
                    .await
                    .expect("Failed to send the game code to the game.");

                // insert a game ws (sender) handle for this game code, so that our Twilio handler can reference it
                games.insert(game_code.clone(), game_sender);
            }

            while let Some(Ok(msg)) = game_reader.next().await {
                if let Message::Close(_) = Message::from(msg) {
                    let mut games = state.games.lock().await;
                    games.remove(&game_code);
                    let mut game_codes = state.game_codes.lock().await;
                    game_codes.insert(game_code.clone());
                }
            }

            let mut games = state.games.lock().await;
            games.remove(&game_code);
            let mut game_codes = state.game_codes.lock().await;
            game_codes.insert(game_code);
        }
        None => {
            return;
        }
    }
}
