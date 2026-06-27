use mongodb::{
    bson::{doc, Document},
    options::{ClientOptions, IndexOptions},
    Client, Collection, Database, IndexModel,
};
use tracing::{info, warn};

use crate::config::MongoDbConfig;
use crate::models::Book;

#[derive(Clone)]
pub struct MongoDb {
    pub client: Client,
    pub database: Database,
    pub books_collection: Collection<Book>,
}

impl MongoDb {
    pub async fn new(config: &MongoDbConfig) -> Result<Self, mongodb::error::Error> {
        info!("Connecting to MongoDB at: {}", config.uri);
        
        let mut client_options = ClientOptions::parse(&config.uri).await?;
        client_options.app_name = Some("Book Library Backend".to_string());
        
        let client = Client::with_options(client_options)?;
        let database = client.database(&config.database);
        let books_collection = database.collection(&config.books_collection);

        // Create indexes
        Self::create_indexes(&books_collection).await?;

        info!("Successfully connected to MongoDB");
        
        Ok(Self {
            client,
            database,
            books_collection,
        })
    }

    async fn create_indexes(books_collection: &Collection<Book>) -> Result<(), mongodb::error::Error> {
        // Create unique index on UUID
        let uuid_index = IndexModel::builder()
            .keys(doc! { "uuid": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build();

        // Create text index for search
        let text_index = IndexModel::builder()
            .keys(doc! { 
                "title": "text",
                "author": "text",
                "description": "text",
                "tags": "text"
            })
            .build();

        // Create compound index for genre and availability
        let genre_availability_index = IndexModel::builder()
            .keys(doc! { "genre": 1, "available_copies": -1 })
            .build();

        // Create index for publication year
        let publication_year_index = IndexModel::builder()
            .keys(doc! { "publication_year": -1 })
            .build();

        let indexes = vec![
            uuid_index,
            text_index,
            genre_availability_index,
            publication_year_index,
        ];

        for index in indexes {
            match books_collection.create_index(index).await {
                Ok(_) => info!("Index created successfully"),
                Err(e) => warn!("Failed to create index: {}", e),
            }
        }

        Ok(())
    }

    pub async fn health_check(&self) -> Result<bool, mongodb::error::Error> {
        self.database
            .run_command(doc! { "ping": 1 })
            .await
            .map(|_| true)
    }
}