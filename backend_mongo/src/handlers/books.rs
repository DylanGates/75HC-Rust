use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::{Book, CreateBook, UpdateBook, BookSearch, BookPagination};
use crate::repository::BookRepository;

#[derive(Clone)]
pub struct AppState {
    pub book_repo: BookRepository,
}

#[derive(Debug, Deserialize)]
pub struct BookIdPath {
    pub id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub title: Option<String>,
    pub author: Option<String>,
    pub genre: Option<String>,
    pub isbn: Option<String>,
    pub tags: Option<String>,
    pub year_min: Option<i32>,
    pub year_max: Option<i32>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

// GET /books - List all books with pagination
pub async fn list_books(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> AppResult<Json<Vec<Book>>> {
    let pagination = BookPagination {
        page: params.page.unwrap_or(1),
        page_size: params.page_size.unwrap_or(20),
    };

    let books = if params.title.is_some() || params.author.is_some() || params.genre.is_some() {
        let search = BookSearch {
            title: params.title,
            author: params.author,
            genre: params.genre.and_then(|g| serde_json::from_str(&format!("\"{}\"", g)).ok()),
            isbn: params.isbn,
            tags: params.tags.map(|t| t.split(',').map(|s| s.trim().to_string()).collect()),
            publication_year_min: params.year_min,
            publication_year_max: params.year_max,
        };
        state.book_repo.search(search, pagination).await?
    } else {
        state.book_repo.get_all(pagination).await?
    };

    Ok(Json(books))
}

// GET /books/popular - Get popular books
pub async fn get_popular_books(
    State(state): State<AppState>,
    Query(params): Query<PopularBooksParams>,
) -> AppResult<Json<Vec<Book>>> {
    let limit = params.limit.unwrap_or(10);
    let books = state.book_repo.get_popular_books(limit).await?;
    Ok(Json(books))
}

#[derive(Debug, Deserialize)]
pub struct PopularBooksParams {
    pub limit: Option<u32>,
}

// GET /books/:id - Get a single book by UUID
pub async fn get_book(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Book>> {
    // Increment view count (fire and forget)
    let _ = state.book_repo.increment_view_count(id).await;
    
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

    if input.author.trim().is_empty() {
        return Err(AppError::BadRequest("Author cannot be empty".to_string()));
    }

    if input.total_copies.unwrap_or(0) < 0 {
        return Err(AppError::BadRequest("Total copies must be non-negative".to_string()));
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

    // Validate author if provided
    if let Some(ref author) = input.author {
        if author.trim().is_empty() {
            return Err(AppError::BadRequest("Author cannot be empty".to_string()));
        }
    }

    // Validate copies
    if let Some(available) = input.available_copies {
        if available < 0 {
            return Err(AppError::BadRequest("Available copies must be non-negative".to_string()));
        }
    }

    if let Some(total) = input.total_copies {
        if total < 0 {
            return Err(AppError::BadRequest("Total copies must be non-negative".to_string()));
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
        Err(AppError::NotFound(format!("Book with UUID {} not found", id)))
    }
}