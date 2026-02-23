mod config;
mod handlers;
mod models;
mod utils;

use axum::{routing::{get, post}, Router};
use hyper_util::rt::TokioIo;
use hyper_util::server::conn::auto::Builder;
use std::{collections::HashMap, net::SocketAddr, sync::{Arc, RwLock}};
use tower::Service;
use tower_http::services::ServeDir;
use tracing::{error, info};

use config::Config;
use handlers::{create_challenge, submit_answer};
use models::{AppState, AppStateData};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("botcaptcha=debug,tower_http=debug")
        .init();

    // Load configuration
    let config = match Config::from_file("config.toml") {
        Ok(cfg) => {
            info!("Loaded configuration from config.toml");
            Arc::new(cfg)
        }
        Err(e) => {
            error!("Failed to load config.toml: {}", e);
            info!("Using default configuration");
            let default_config = Config::default();
            // Optionally save default config for user reference
            if let Err(save_err) = default_config.save_to_file("config.toml") {
                error!("Failed to save default config: {}", save_err);
            } else {
                info!("Created default config.toml file");
            }
            Arc::new(default_config)
        }
    };

    // Create shared state with config
    let state: AppState = Arc::new(RwLock::new(AppStateData {
        challenges: HashMap::new(),
        config: config.clone(),
    }));

    // Build router
    let app = Router::new()
        .route("/api/challenge", get(create_challenge))
        .route("/api/submit", post(submit_answer))
        .nest_service("/", ServeDir::new("static"))
        .with_state(state);

    // Start server using config
    let addr = SocketAddr::from((
        config.server.host.parse::<std::net::IpAddr>().unwrap(),
        config.server.port,
    ));
    info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    loop {
        let (socket, _remote_addr) = listener.accept().await.unwrap();
        let tower_service = app.clone();

        tokio::spawn(async move {
            let socket = TokioIo::new(socket);
            let hyper_service = hyper::service::service_fn(move |request| {
                tower_service.clone().call(request)
            });

            if let Err(err) = Builder::new(hyper_util::rt::TokioExecutor::new())
                .serve_connection(socket, hyper_service)
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
