# Book Library Backend

A modern Rust backend for a book library system using MongoDB and Redis with caching.

## Features

- **MongoDB Integration**: Full CRUD operations with proper indexing
- **Redis Caching**: Intelligent caching with TTL support
- **RESTful API**: Clean API design with proper HTTP status codes
- **Error Handling**: Comprehensive error handling with proper responses
- **Logging**: Structured logging with tracing
- **Docker Support**: Easy deployment with Docker Compose
- **Health Checks**: Service health monitoring
- **CORS Support**: Configurable CORS headers

## Architecture

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Client    │────▶│  API Layer  │────▶│  Handlers   │
└─────────────┘     └─────────────┘     └─────────────┘
                                                       │
                       ┌─────────────┐     ┌─────────────┐
                       │   Redis     │◀────│ Repository  │
                       │   Cache     │     │   Layer     │
                       └─────────────┘     └─────────────┘
                                                       │
                                               ┌─────────────┐
                                               │  MongoDB    │
                                               │  Database   │
                                               └─────────────┘
```

## API Endpoints

### Health Check
- `GET /health` - Check service health

### Books
- `GET /books` - List all books with pagination and search
- `POST /books` - Create a new book
- `GET /books/popular` - Get popular books
- `GET /books/:id` - Get a specific book by UUID
- `PATCH /books/:id` - Update a book
- `DELETE /books/:id` - Delete a book

## Quick Start

### Using Docker Compose (Recommended)

1. Start the services:
```bash
docker-compose up -d
```

2. The API will be available at `http://localhost:3000`

### Local Development

1. Install dependencies:
```bash
cargo build
```

2. Start MongoDB and Redis:
```bash
# Using Docker
docker run -d -p 27017:27017 --name mongodb mongo:7
docker run -d -p 6379:6379 --name redis redis:7-alpine
```

3. Copy environment file:
```bash
cp .env.example .env
```

4. Run the application:
```bash
cargo run
```

## Configuration

The application can be configured using environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `SERVER_HOST` | Server host address | `0.0.0.0` |
| `SERVER_PORT` | Server port | `3000` |
| `MONGODB_URI` | MongoDB connection URI | `mongodb://localhost:27017` |
| `MONGODB_DATABASE` | MongoDB database name | `book_library` |
| `REDIS_URL` | Redis connection URL | `redis://localhost:6379` |
| `REDIS_TTL_SECONDS` | Cache TTL in seconds | `3600` |
| `RUST_LOG` | Logging configuration | `book_library=debug,tower_http=debug` |

## Book Model

```json
{
  "id": "507f1f77bcf86cd799439011",
  "uuid": "550e8400-e29b-41d4-a716-446655440000",
  "title": "The Rust Programming Language",
  "description": "A comprehensive guide to Rust programming",
  "author": "Steve Klabnik and Carol Nichols",
  "genre": "Technology",
  "isbn": "978-1593278281",
  "publication_year": 2019,
  "page_count": 552,
  "language": "en",
  "publisher": "No Starch Press",
  "cover_image_url": "https://example.com/cover.jpg",
  "rating": 4.8,
  "tags": ["programming", "rust", "systems"],
  "available_copies": 5,
  "total_copies": 10,
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

## Example Requests

### Create a Book
```bash
curl -X POST http://localhost:3000/books \
  -H "Content-Type: application/json" \
  -d '{
    "title": "The Rust Programming Language",
    "description": "A comprehensive guide to Rust programming",
    "author": "Steve Klabnik and Carol Nichols",
    "genre": "Technology",
    "isbn": "978-1593278281",
    "publication_year": 2019,
    "page_count": 552,
    "language": "en",
    "publisher": "No Starch Press",
    "tags": ["programming", "rust", "systems"],
    "total_copies": 10
  }'
```

### Search Books
```bash
# Search by title
curl "http://localhost:3000/books?title=rust"

# Search by author
curl "http://localhost:3000/books?author=Steve"

# Search by genre
curl "http://localhost:3000/books?genre=Technology"

# Combined search with pagination
curl "http://localhost:3000/books?title=rust&page=1&page_size=10"
```

### Get Popular Books
```bash
curl "http://localhost:3000/books/popular?limit=5"
```

## Development

### Running Tests
```bash
cargo test
```

### Building for Production
```bash
cargo build --release
```

# Create a book
curl -X POST http://localhost:3000/books \
  -H "Content-Type: application/json" \
  -d '{"title": "Rust Programming", "author": "Steve Klabnik", "genre": "Technology"}'

# Search books
curl "http://localhost:3000/books?title=rust&page=1"

# Get book by ID
curl "http://localhost:3000/books/550e8400-e29b-41d4-a716-446655440000"

# Health check
curl "http://localhost:3000/health"

### API Documentation
The API follows RESTful conventions and returns proper HTTP status codes:

- `200 OK` - Successful GET requests
- `201 Created` - Successful POST requests
- `204 No Content` - Successful DELETE requests
- `400 Bad Request` - Invalid input data
- `404 Not Found` - Resource not found
- `500 Internal Server Error` - Server errors

## Performance Features

- **Redis Caching**: Frequently accessed books are cached in Redis
- **MongoDB Indexing**: Optimized queries with proper indexing
- **Connection Pooling**: Efficient database connection management
- **Async/Await**: Non-blocking I/O operations
- **Request Tracing**: Detailed request/response logging

## Monitoring

The application provides health check endpoints and structured logging for monitoring:

- `/health` - Service health status
- Structured logs with request tracing
- Database connection status
- Cache hit/miss statistics

## License

This project is licensed under the MIT License.