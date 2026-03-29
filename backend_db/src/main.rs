use axum::Router;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tower_http::trace::TraceLayer;
use tracing::Span;
use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;
use tower_governor::{GovernorConfig, GovernorLayer};
use std::time::Duration;

mod db;
mod error;
mod handlers;
mod models;
mod routes;

use db_pool::create_pool;
use db_pool::Pool;
use db::books::BookRepository;
use handlers::books::AppState;
use routes::books::book_routes;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing for structured logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "book_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Get database URL from environment
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    // Create database connection pool
    tracing::info!("Connecting to database...");
    let pool = create_pool(&database_url).await?;
    tracing::info!("Database connection established");

    // Run migrations
    tracing::info!("Running database migrations...");
    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("Migrations complete");

    // Create application state
    let state = AppState {
        book_repo: BookRepository::new(pool),
    };

    // Configure rate limiting: 60 requests per minute with burst of 10
    let governor_config = GovernorConfig::builder()
        .per_second(60) // 60 requests per minute
        .burst_size(10) // Allow burst of 10 requests
        .finish()
        .unwrap();

    // Build the router
    let app = Router::new()
        .merge(book_routes())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &axum::http::Request<_>| {
                    tracing::info_span!(
                        "http_request",
                        method = %request.method(),
                        uri = %request.uri(),
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
        .layer(GovernorLayer {
            config: std::sync::Arc::new(governor_config),
        })
        .with_state(state);

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}