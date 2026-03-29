use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::db::books::BookRepository;
use crate::error::{AppError, AppResult};
use crate::models::book::{CreateBook, Book, UpdateBook, DeleteBook};

// AppState holds shared application state
// Clone is cheap because PgPool uses Arc internally
#[derive(Clone)]
pub struct AppState {
    pub book_repo: BookRepository,
}

// GET /books - List all books
pub async fn list_books(State(state): State<AppState>) -> AppResult<Json<Vec<Book>>> {
    let books = state.book_repo.get_all().await?;
    Ok(Json(books))
}

// GET /books/:id - Get a single book
pub async fn get_book(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Book>> {
    let book = state.book_repo.get_by_id(id).await?;
    Ok(Json(book))
}

// POST /books - Create a new book
pub async fn create_book(
    State(state): State<AppState>,
    Json(input): Json<CreateBook>,
) -> AppResult<(StatusCode, Json<Book>)> {
    // Validate input
    if input.title.trim().is_empty() {
        return Err(AppError::BadRequest("Title cannot be empty".to_string()));
    }

    let book = state.book_repo.create(input).await?;

    // Return 201 Created with the new book
    Ok((StatusCode::CREATED, Json(book)))
}

// PATCH /books/:id - Update a book
pub async fn update_book(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateBook>,
) -> AppResult<Json<Book>> {
    // Validate title if provided
    if let Some(ref title) = input.title {
        if title.trim().is_empty() {
            return Err(AppError::BadRequest("Title cannot be empty".to_string()));
        }
    }

    let book = state.book_repo.update(id, input).await?;
    Ok(Json(book))      
}

// DELETE /books/:id - Delete a book
pub async fn delete_book(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    let deleted = state.book_repo.delete(id).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound(format!("Book with id {} not found", id)))
    }
}