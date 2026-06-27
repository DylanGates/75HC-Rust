use std::net::SocketAddr;

use axum::{routing::{get, patch, delete}, Router};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::Span;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;
use crate::db::{MongoDb, RedisCache};
use crate::handlers::{AppState, HealthState};
use crate::repository::BookRepository;

mod config;
mod db;
mod error;
mod handlers;
mod models;
mod repository;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "book_library=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    dotenvy::dotenv().ok();
    let config = Config::from_env().unwrap_or_default();
    
    tracing::info!("Starting Book Library Backend");
    tracing::info!("Configuration: {:?}", config);

    // Connect to MongoDB
    tracing::info!("Connecting to MongoDB...");
    let mongodb = MongoDb::new(&config.mongodb).await?;
    tracing::info!("MongoDB connection established");

    // Connect to Redis
    tracing::info!("Connecting to Redis...");
    let redis = RedisCache::new(&config.redis).await?;
    tracing::info!("Redis connection established");

    // Create repositories
    let book_repo = BookRepository::new(mongodb.clone(), redis.clone());

    // Create application states
    let app_state = AppState {
        book_repo: book_repo.clone(),
    };

    let health_state = HealthState {
        mongodb: mongodb.clone(),
        redis: redis.clone(),
    };

    // Build routes
    let app = Router::new()
        // Health check
        .route("/health", get(handlers::health_check))
        .with_state(health_state)
        
        // Book routes
        .route("/books", get(handlers::list_books).post(handlers::create_book))
        .route("/books/popular", get(handlers::get_popular_books))
        .route("/books/:id", get(handlers::get_book))
        .route("/books/:id", patch(handlers::update_book))
        .route("/books/:id", delete(handlers::delete_book))
        .with_state(app_state);

    // Add middleware layers
    let app = app
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &axum::http::Request<_>| {
                    tracing::info_span!(
                        "http_request",
                        method = %request.method(),
                        uri = %request.uri(),
                        version = ?request.version(),
                    )
                })
                .on_response(|response: &axum::http::Response<_>, latency: std::time::Duration, _span: &Span| {
                    tracing::info!(
                        status = %response.status(),
                        latency = ?latency,
                        "response"
                    );
                }),
        )
        .layer(CorsLayer::permissive()); // Configure CORS as needed

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}