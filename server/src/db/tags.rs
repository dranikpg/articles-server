use super::Db;
use rocket::serde::Serialize;

// ========================== TYPES =======================

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct Tag {
    pub id: i32,
    pub name: String,
    pub num_articles: i64,
}

#[derive(Debug, thiserror::Error, Serialize)]
pub enum TagError {
    #[error("Internal")]
    Internal(
        #[from]
        #[source]
        #[serde(skip)]
        sqlx::Error,
    ),
}

// ========================== FUNCTIONS ===================

pub async fn list(db: &Db, user: i32) -> Result<Vec<Tag>, TagError> {
    Ok(sqlx::query_as(
        "SELECT t.id, t.name, 
        (SELECT COUNT(article_id) FROM article_tags at WHERE at.tag_id = t.id) as num_articles
        FROM tags t
        WHERE t.user_id = $1
        ORDER BY num_articles DESC")
        .bind(user)
        .fetch_all(db)
        .await?)
}

pub async fn update_article_tags(
    db: &Db,
    user: i32,
    article: i32,
    tags: &[&str],
) -> Result<(), sqlx::Error> {
    // delete all tag mappings
    sqlx::query("DELETE FROM article_tags WHERE article_id = $1")
        .bind(article)
        .execute(db)
        .await?;
    // insert all tags TODO: Improve!
    for tag in tags {
        sqlx::query("INSERT INTO tags (user_id, name) VALUES ($1, LOWER($2))
            ON CONFLICT DO NOTHING")
            .bind(user)
            .bind(tag)
            .execute(db)
            .await?;
        let id: i32 = sqlx::query_scalar("SELECT id FROM tags WHERE user_id = $1 AND name = LOWER($2)")
            .bind(user)
            .bind(tag)
            .fetch_one(db)
            .await?;
        sqlx::query("INSERT INTO article_tags (article_id, tag_id) VALUES ($1, $2)")
            .bind(article)
            .bind(id)
            .execute(db)
            .await?;
    }
    // remove orphans
    sqlx::query("DELETE FROM tags t WHERE 
        user_id = $1 AND
        (SELECT COUNT(article_id) FROM article_tags WHERE tag_id = t.id) = 0")
        .bind(user)
        .execute(db)
        .await?;
    Ok(())
}
