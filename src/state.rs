use crate::message::Message;
use futures::lock::Mutex;
use std::collections::{HashMap, HashSet};

pub struct State {
    pub deepgram_url: String,
    pub api_key: String,
    pub twilio_phone_number: String,
    pub games: Mutex<HashMap<String, GameTwilioTxs>>,
    pub game_codes: Mutex<HashSet<String>>,
}

pub struct GameTwilioTxs {
    pub game_tx: async_channel::Sender<Message>,
    pub twilio_tx: Option<async_channel::Sender<Message>>,
}
