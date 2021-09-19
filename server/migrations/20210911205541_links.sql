CREATE TABLE IF NOT EXISTS links (
    id SERIAL PRIMARY KEY,
    user_id SERIAL NOT NULL REFERENCES users (id),
    article_id SERIAL NOT NULL REFERENCES articles (id),

    fresh BOOLEAN DEFAULT TRUE,
    url TEXT NOT NULL,

    title TEXT, 
    content TEXT,
    language regconfig,
    search_vector tsvector,
    screenshot BYTEA
);