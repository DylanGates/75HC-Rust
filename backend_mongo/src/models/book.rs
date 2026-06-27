use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Genre {
    Action,
    Adventure,
    Drama,
    Horror,
    ScienceFiction,
    Mystery,
    Romance,
    Fantasy,
    Biography,
    History,
}

impl fmt::Display for Genre {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Genre::Action => write!(f, "action"),
            Genre::Adventure => write!(f, "adventure"),
            Genre::Drama => write!(f, "drama"),
            Genre::Horror => write!(f, "horror"),
            Genre::ScienceFiction => write!(f, "science_fiction"),
            Genre::Mystery => write!(f, "mystery"),
            Genre::Romance => write!(f, "romance"),
            Genre::Fantasy => write!(f, "fantasy"),
            Genre::Biography => write!(f, "biography"),
            Genre::History => write!(f, "history"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub uuid: Uuid,
    pub title: String,
    pub description: String,
    pub author: String,
    pub genre: Genre,
    pub isbn: Option<String>,
    pub publication_year: Option<i32>,
    pub page_count: Option<i32>,
    pub language: String,
    pub publisher: Option<String>,
    pub cover_image_url: Option<String>,
    pub rating: Option<f64>,
    pub tags: Vec<String>,
    pub available_copies: i32,
    pub total_copies: i32,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateBook {
    pub title: String,
    pub description: String,
    pub author: String,
    pub genre: Genre,
    pub isbn: Option<String>,
    pub publication_year: Option<i32>,
    pub page_count: Option<i32>,
    pub language: Option<String>,
    pub publisher: Option<String>,
    pub cover_image_url: Option<String>,
    pub tags: Option<Vec<String>>,
    pub total_copies: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBook {
    pub title: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub genre: Option<Genre>,
    pub isbn: Option<String>,
    pub publication_year: Option<i32>,
    pub page_count: Option<i32>,
    pub language: Option<String>,
    pub publisher: Option<String>,
    pub cover_image_url: Option<String>,
    pub rating: Option<f64>,
    pub tags: Option<Vec<String>>,
    pub available_copies: Option<i32>,
    pub total_copies: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct BookSearch {
    pub title: Option<String>,
    pub author: Option<String>,
    pub genre: Option<Genre>,
    pub isbn: Option<String>,
    pub tags: Option<Vec<String>>,
    pub publication_year_min: Option<i32>,
    pub publication_year_max: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct BookPagination {
    pub page: u32,
    pub page_size: u32,
}

impl Default for BookPagination {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 20,
        }
    }
}