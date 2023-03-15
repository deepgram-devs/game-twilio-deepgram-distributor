use axum::{routing::get, Extension, Router};
use axum_server::tls_rustls::RustlsConfig;
use futures::lock::Mutex;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use structopt::StructOpt;

mod audio;
mod config;
mod deepgram_response;
mod handlers;
mod message;
mod state;
mod twilio_response;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(parse(from_os_str))]
    config: Option<PathBuf>,
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();

    let proxy_url = std::env::var("PROXY_URL").unwrap_or_else(|_| "127.0.0.1:5000".to_string());

    let deepgram_url = std::env::var("DEEPGRAM_URL").unwrap_or_else(|_| {
        "wss://api.deepgram.com/v1/listen?encoding=mulaw&sample_rate=8000&numerals=true".to_string()
    });

    let api_key =
        std::env::var("DEEPGRAM_API_KEY").expect("Using this server requires a Deepgram API Key.");

    let twilio_phone_number = std::env::var("TWILIO_PHONE_NUMBER")
        .expect("Using this server requires a Twilio phone number.");

    let cert_pem = std::env::var("CERT_PEM").ok();
    let key_pem = std::env::var("KEY_PEM").ok();

    let config = match (cert_pem, key_pem) {
        (Some(cert_pem), Some(key_pem)) => Some(
            RustlsConfig::from_pem_file(cert_pem, key_pem)
                .await
                .expect("Failed to make RustlsConfig from cert/key pem files."),
        ),
        (None, None) => None,
        _ => {
            panic!("Failed to start - invalid cert/key.")
        }
    };

    let game_codes = match opt.config.and_then(|config_path| {
        let config_file = std::fs::File::open(config_path).expect("Failed to open config file.");
        let config: config::Config =
            serde_json::from_reader(config_file).expect("Failed to read config file.");

        if config.game_codes.is_empty() {
            None
        } else {
            Some(config.game_codes)
        }
    }) {
        Some(game_codes) => HashSet::from_iter(game_codes.iter().cloned()),
        None => {
            let mut game_codes = HashSet::new();

            for number in 0..100 {
                game_codes.insert(number.to_string());
            }
            game_codes
        }
    };

    let state = Arc::new(state::State {
        deepgram_url,
        api_key,
        twilio_phone_number,
        games: Mutex::new(HashMap::new()),
        game_codes: Mutex::new(game_codes),
    });

    let app = Router::new()
        .route("/twilio", get(handlers::twilio::twilio_handler))
        .route("/game", get(handlers::game::game_handler))
        .layer(Extension(state));

    match config {
        Some(config) => {
            axum_server::bind_rustls(proxy_url.parse().unwrap(), config)
                .serve(app.into_make_service())
                .await
                .unwrap();
        }
        None => {
            axum_server::bind(proxy_url.parse().unwrap())
                .serve(app.into_make_service())
                .await
                .unwrap();
        }
    }
}
