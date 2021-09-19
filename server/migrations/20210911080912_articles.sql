CREATE TABLE IF NOT EXISTS articles (
    id SERIAL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES users (id),

    title TEXT NOT NULL,
    content TEXT NOT NULL,
    raw_text TEXT NOT NULL,
    preview TEXT NOT NULL,

    language regconfig,
    search_vector tsvector 
        GENERATED ALWAYS AS (to_tsvector(language, LOWER(title) || ' ' || LOWER(raw_text))) STORED,

    created_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- index

CREATE INDEX articles_search ON articles USING GIN (search_vector);

-- Trigger for updated_on

CREATE OR REPLACE FUNCTION articles_trigger_edited()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_on = CURRENT_TIMESTAMP;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS set_timestamp on public.articles;

CREATE TRIGGER set_timestamp
BEFORE UPDATE ON articles
FOR EACH ROW
EXECUTE FUNCTION articles_trigger_edited();
