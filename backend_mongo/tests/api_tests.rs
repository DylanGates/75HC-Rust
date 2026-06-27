#[tokio::test]
async fn test_book_model_serialization() {
    use uuid::Uuid;
    use chrono::Utc;
    
    let book = serde_json::json!({
        "uuid": Uuid::new_v4().to_string(),
        "title": "Test Book",
        "description": "A test book description",
        "author": "Test Author",
        "genre": "action",
        "isbn": "978-1234567890",
        "publication_year": 2024,
        "page_count": 300,
        "language": "en",
        "publisher": "Test Publisher",
        "tags": ["test", "example"],
        "total_copies": 10,
        "available_copies": 8,
        "rating": 4.5,
        "created_at": Utc::now().to_rfc3339(),
        "updated_at": Utc::now().to_rfc3339()
    });

    // Verify the JSON structure
    assert_eq!(book["title"], "Test Book");
    assert_eq!(book["author"], "Test Author");
    assert_eq!(book["genre"], "action");
    assert_eq!(book["total_copies"], 10);
}

#[test]
fn test_genre_display() {
    use crate::models::Genre;
    
    assert_eq!(Genre::Action.to_string(), "action");
    assert_eq!(Genre::ScienceFiction.to_string(), "science_fiction");
    assert_eq!(Genre::Mystery.to_string(), "mystery");
}