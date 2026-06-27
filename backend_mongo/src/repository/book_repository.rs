use futures_util::stream::TryStreamExt;
use mongodb::{
    bson::{doc},
    options::FindOptions,
};
use tracing::info;

use crate::db::{MongoDb, RedisCache};
use crate::error::{AppError, AppResult};
use crate::models::{Book, CreateBook, UpdateBook, BookSearch, BookPagination};

#[derive(Clone)]
pub struct BookRepository {
    mongodb: MongoDb,
    redis: RedisCache,
}

impl BookRepository {
    pub fn new(mongodb: MongoDb, redis: RedisCache) -> Self {
        Self { mongodb, redis }
    }

    pub async fn create(&self, input: CreateBook) -> AppResult<Book> {
        let book = Book {
            id: None,
            uuid: uuid::Uuid::new_v4(),
            title: input.title,
            description: input.description,
            author: input.author,
            genre: input.genre,
            isbn: input.isbn,
            publication_year: input.publication_year,
            page_count: input.page_count,
            language: input.language.unwrap_or_else(|| "en".to_string()),
            publisher: input.publisher,
            cover_image_url: input.cover_image_url,
            rating: None,
            tags: input.tags.unwrap_or_default(),
            available_copies: input.total_copies.unwrap_or(1),
            total_copies: input.total_copies.unwrap_or(1),
            created_at: mongodb::bson::DateTime::now(),
            updated_at: mongodb::bson::DateTime::now(),
        };

        let result = self.mongodb.books_collection.insert_one(&book).await?;
        let inserted_id = result.inserted_id.as_object_id().unwrap();
        
        // Cache the new book
        let mut cached_book = book.clone();
        cached_book.id = Some(inserted_id);
        let cache_key = format!("book:{}", cached_book.uuid);
        self.redis.set(&cache_key, &cached_book).await?;

        info!("Created book with UUID: {}", cached_book.uuid);
        Ok(cached_book)
    }

    pub async fn get_by_id(&self, uuid: uuid::Uuid) -> AppResult<Book> {
        let cache_key = format!("book:{}", uuid);
        
        // Try to get from cache first
        if let Some(cached_book) = self.redis.get::<Book>(&cache_key).await? {
            info!("Cache hit for book: {}", uuid);
            return Ok(cached_book);
        }

        info!("Cache miss for book: {}, querying MongoDB", uuid);
        
        let book = self.mongodb.books_collection
            .find_one(doc! { "uuid": uuid.to_string() })
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Book with UUID {} not found", uuid)))?;

        // Cache the result
        self.redis.set(&cache_key, &book).await?;
        
        Ok(book)
    }

    pub async fn get_all(&self, pagination: BookPagination) -> AppResult<Vec<Book>> {
        let options = FindOptions::builder()
            .skip(Some(((pagination.page - 1) * pagination.page_size) as u64))
            .limit(Some(pagination.page_size as i64))
            .sort(doc! { "created_at": -1 })
            .build();

        let mut cursor = self.mongodb.books_collection
            .find(doc! {})
            .with_options(options)
            .await?;

        let mut books = Vec::new();
        while cursor.advance().await? {
            if let Some(book) = cursor.deserialize_current().ok() {
                books.push(book);
            }
        }
        Ok(books)
    }

    pub async fn search(&self, search: BookSearch, pagination: BookPagination) -> AppResult<Vec<Book>> {
        let mut filter = mongodb::bson::Document::new();

        if let Some(title) = &search.title {
            filter.insert("title", doc! { "$regex": title, "$options": "i" });
        }

        if let Some(author) = &search.author {
            filter.insert("author", doc! { "$regex": author, "$options": "i" });
        }

        if let Some(genre) = &search.genre {
            filter.insert("genre", genre.to_string());
        }

        if let Some(isbn) = &search.isbn {
            filter.insert("isbn", isbn);
        }

        if let Some(tags) = &search.tags {
            filter.insert("tags", doc! { "$in": tags });
        }

        if let Some(year_min) = search.publication_year_min {
            filter.insert("publication_year", doc! { "$gte": year_min });
        }

        if let Some(year_max) = search.publication_year_max {
            filter.insert("publication_year", doc! { "$lte": year_max });
        }

        let options = FindOptions::builder()
            .skip(Some(((pagination.page - 1) * pagination.page_size) as u64))
            .limit(Some(pagination.page_size as i64))
            .sort(doc! { "created_at": -1 })
            .build();

        let mut cursor = self.mongodb.books_collection
            .find(filter)
            .with_options(options)
            .await?;

        let mut books = Vec::new();
        while cursor.advance().await? {
            if let Some(book) = cursor.deserialize_current().ok() {
                books.push(book);
            }
        }
        Ok(books)
    }

    pub async fn update(&self, uuid: uuid::Uuid, input: UpdateBook) -> AppResult<Book> {
        let mut update_doc = mongodb::bson::Document::new();
        
        if let Some(title) = input.title {
            update_doc.insert("title", title);
        }
        
        if let Some(description) = input.description {
            update_doc.insert("description", description);
        }
        
        if let Some(author) = input.author {
            update_doc.insert("author", author);
        }
        
        if let Some(genre) = input.genre {
            update_doc.insert("genre", genre.to_string());
        }
        
        if let Some(isbn) = input.isbn {
            update_doc.insert("isbn", isbn);
        }
        
        if let Some(publication_year) = input.publication_year {
            update_doc.insert("publication_year", publication_year);
        }
        
        if let Some(page_count) = input.page_count {
            update_doc.insert("page_count", page_count);
        }
        
        if let Some(language) = input.language {
            update_doc.insert("language", language);
        }
        
        if let Some(publisher) = input.publisher {
            update_doc.insert("publisher", publisher);
        }
        
        if let Some(cover_image_url) = input.cover_image_url {
            update_doc.insert("cover_image_url", cover_image_url);
        }
        
        if let Some(rating) = input.rating {
            update_doc.insert("rating", rating);
        }
        
        if let Some(tags) = input.tags {
            update_doc.insert("tags", tags);
        }
        
        if let Some(available_copies) = input.available_copies {
            update_doc.insert("available_copies", available_copies);
        }
        
        if let Some(total_copies) = input.total_copies {
            update_doc.insert("total_copies", total_copies);
        }

        update_doc.insert("updated_at", mongodb::bson::DateTime::now());

        let result = self.mongodb.books_collection
            .find_one_and_update(
                doc! { "uuid": uuid.to_string() },
                doc! { "$set": update_doc },
            )
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Book with UUID {} not found", uuid)))?;

        // Update cache
        let cache_key = format!("book:{}", uuid);
        self.redis.set(&cache_key, &result).await?;

        info!("Updated book with UUID: {}", uuid);
        Ok(result)
    }

    pub async fn delete(&self, uuid: uuid::Uuid) -> AppResult<bool> {
        let result = self.mongodb.books_collection
            .delete_one(doc! { "uuid": uuid.to_string() })
            .await?;

        if result.deleted_count > 0 {
            // Remove from cache
            let cache_key = format!("book:{}", uuid);
            self.redis.delete(&cache_key).await?;
            
            info!("Deleted book with UUID: {}", uuid);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn increment_view_count(&self, uuid: uuid::Uuid) -> AppResult<i64> {
        let cache_key = format!("book:views:{}", uuid);
        let view_count = self.redis.increment(&cache_key).await?;
        Ok(view_count)
    }

    pub async fn get_popular_books(&self, limit: u32) -> AppResult<Vec<Book>> {
        // This is a simplified implementation
        // In a real app, you'd track views and ratings
        let options = FindOptions::builder()
            .limit(Some(limit as i64))
            .sort(doc! { "rating": -1, "created_at": -1 })
            .build();

        let mut cursor = self.mongodb.books_collection
            .find(doc! {})
            .with_options(options)
            .await?;

        let mut books = Vec::new();
        while cursor.advance().await? {
            if let Some(book) = cursor.deserialize_current().ok() {
                books.push(book);
            }
        }
        Ok(books)
    }
}