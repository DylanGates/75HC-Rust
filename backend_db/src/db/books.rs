use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::book::{CreateBook, Book, BookPagination, BookSearch, BookUpdate, BookDelete};

// BookRepository encapsulates all database operations for books
// Using a struct allows for easier testing with mocks
pub struct BookRepository {
    pool: PgPool,
}

impl BookRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // Retrieve all books from the database
    // In production, you'd add pagination here
    pub async fn get_all(&self) -> AppResult<Vec<Book>> {
        let books = sqlx::query_as::<_, Book>(
            r#"
            SELECT id, title, description, author, genre, publication_date, created_at, updated_at
            FROM books
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(books)
    }

    // Get a single book by ID  
    pub async fn get_by_id(&self, id: Uuid) -> AppResult<Book> {
        sqlx::query_as::<_, Book>(
            r#"
            SELECT id, title, description, author, genre, publication_date, created_at, updated_at
            FROM books
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Book with id {} not found", id)))
    }

    // Create a new book
    // UUID and timestamps are generated server-side for consistency
    pub async fn create(&self, input: CreateBook) -> AppResult<Book> {
        let now = Utc::now();
        let id = Uuid::new_v4();

        let book = sqlx::query_as::<_, Book>(
            r#"
            INSERT INTO books (id, title, description, author, genre, publication_date, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, title, description, author, genre, publication_date, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(&input.title)
        .bind(&input.description)
        .bind(&input.author)
        .bind(&input.genre)
        .bind(&input.publication_date)  
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(book)
    }

    // Update an existing book
    // Uses COALESCE to only update provided fields
    pub async fn update(&self, id: Uuid, input: UpdateBook) -> AppResult<Book> {
        let now = Utc::now();

        let book = sqlx::query_as::<_, Book>(
            r#"
            UPDATE books
            SET
                title = COALESCE($2, title),
                description = COALESCE($3, description),
                author = COALESCE($4, author),
                genre = COALESCE($5, genre),
                publication_date = COALESCE($6, publication_date),
                updated_at = $7
            WHERE id = $1
            RETURNING id, title, description, author, genre, publication_date, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(&input.title)
        .bind(&input.description)
        .bind(&input.author)
        .bind(&input.genre)
        .bind(&input.publication_date)
        .bind(now)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Book with id {} not found", id)));

        Ok(book)    
    }

    // Delete a book by ID
    // Returns true if a book was deleted, false if not found
    pub async fn delete(&self, id: Uuid) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM books WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}