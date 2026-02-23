mod config;
mod handlers;
mod models;
mod utils;

use axum::{http::Method, routing::{get, post}, Router};
use hyper_util::rt::TokioIo;
use hyper_util::server::conn::auto::Builder;
use std::{collections::{HashMap, HashSet}, net::SocketAddr, sync::{Arc, RwLock}};
use tower::Service;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};
use tracing::{error, info};

use config::Config;
use handlers::{create_challenge, submit_answer, verify_token};
use models::{AppState, AppStateData};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("botcaptcha=debug,tower_http=debug")
        .init();

    // Load configuration — config.toml must exist in the working directory
    let config = Config::from_file("config.toml").unwrap_or_else(|e| {
        error!("Failed to load config.toml: {}", e);
        std::process::exit(1);
    });
    info!("Loaded configuration from config.toml");
    let config = Arc::new(config);

    // Create shared state with config
    let state: AppState = Arc::new(RwLock::new(AppStateData {
        challenges: HashMap::new(),
        used_tokens: HashSet::new(),
        config: config.clone(),
    }));

    // CORS: allow any origin so widget.js embedded on external sites can reach the API
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    // Build router
    let app = Router::new()
        .route("/api/challenge", get(create_challenge))
        .route("/api/submit", post(submit_answer))
        .route("/api/verify", post(verify_token))
        .layer(cors)
        .nest_service("/", ServeDir::new("static"))
        .with_state(state);

    // Start server
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
