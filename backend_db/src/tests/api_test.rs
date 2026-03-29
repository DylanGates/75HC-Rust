use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

// Helper function to create test app
async fn setup_test_app() -> Router {
    todo!("Set up test database and app for books")
}

#[tokio::test]
    async fn test_create_book() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/tasks")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "title": "Test Book",
                        "description": "A test book",
                        "author": "Test Author",
                        "genre": "action"
                    })       
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    // Parse response body and verify fields
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let book: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(book["title"], "Test Book");
    assert_eq!(book["description"], "A test book");
    assert_eq!(book["author"], "Test Author");
    assert_eq!(book["genre"], "action");    
}

#[tokio::test]
    async fn test_create_book_empty_title() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/books")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "title": "",
                        "description": "A book with empty title"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return 400 Bad Request for empty title
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}