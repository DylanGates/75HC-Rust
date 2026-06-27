use axum::{routing::get, Router};

use crate::handlers::{AppState, HealthState};
use crate::handlers::{list_books, get_book, create_book, update_book, delete_book, get_popular_books, health_check};

pub fn create_routes(app_state: AppState, health_state: HealthState) -> Router {
    Router::new()
        // Health check
        .route("/health", get(health_check))
        
        // Book routes
        .route("/books", get(list_books).post(create_book))
        .route("/books/popular", get(get_popular_books))
        .route("/books/:id", get(get_book).patch(update_book).delete(delete_book))
        
        // Layer states
        .with_state(app_state)
}