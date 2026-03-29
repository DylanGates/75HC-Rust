use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "genre", rename_all = "lowercase")]
pub enum Genre {
    Action,
    Adventure,
    Drama,
    Horror,
    ScienceFiction,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Book {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub author: String,
    pub genre: Genre,
    pub publication_date: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct BookCreate {
    pub title: String,
    pub description: String,
    pub author: String,
    pub genre: Genre,
}

#[derive(Debug, Deserialize)]
pub struct BookUpdate {
    pub title: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub genre: Option<Genre>,
}

#[derive(Debug, Deserialize)]
pub struct BookDelete {
    pub id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct BookSearch {
    pub title: Option<String>,
    pub author: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BookPagination {
    pub page: u32,
    pub page_size: u32,
}