-- Create books table
CREATE TABLE IF NOT EXISTS books (
    id UUID PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    author VARCHAR(255) NOT NULL,
    genre VARCHAR(50) NOT NULL,
    publication_date TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create index on title for faster searches
CREATE INDEX idx_books_title ON books(title);

-- Create index on author for faster searches
CREATE INDEX idx_books_author ON books(author);

-- Create index on genre for filtering
CREATE INDEX idx_books_genre ON books(genre);