use axum::{
    routing::{delete, get, patch, post},
    Router,
};

use crate::handlers::books::{
    create_book, delete_book, get_book, list_books, update_book, AppState,
};

// Build the book router with all endpoints
pub fn book_routes() -> Router<AppState> {
    Router::new()
        // Collection routes
        .route("/books", get(list_books))
        .route("/books", post(create_book))
        // Individual resource routes
        .route("/books/:id", get(get_book))
        .route("/books/:id", patch(update_book))
        .route("/books/:id", delete(delete_book))
}