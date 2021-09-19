CREATE TABLE IF NOT EXISTS tags (
    id SERIAL PRIMARY KEY,
    user_id SERIAL REFERENCES users (id),
    name TEXT NOT NULL,
    UNIQUE(user_id, name)
);

CREATE TABLE IF NOT EXISTS article_tags (
    article_id SERIAL REFERENCES articles (id) ON UPDATE CASCADE ON DELETE CASCADE,
    tag_id SERIAL REFERENCES tags (id) ON UPDATE CASCADE,
    CONSTRAINT article_tags_pkey PRIMARY KEY (article_id, tag_id)
)