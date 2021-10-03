use chrono::NaiveDateTime;
use rocket::serde::{Deserialize, Serialize};
use super::links::Analyzer as LinkAnalyzer;
use super::Db;
use crate::utils::extractor;

// ========================== TYPES =======================

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct Article {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub created_on: NaiveDateTime,
    pub updated_on: NaiveDateTime,
}

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct ArticlePreview {
    pub id: i32,
    pub title: String,
    pub preview: String,
    pub tags: Vec<String>,
    pub created_on: NaiveDateTime,
    pub updated_on: NaiveDateTime,
}

#[derive(Deserialize)]
pub struct ArticleInsert<'a> {
    pub id: Option<i32>,
    pub title: &'a str,
    pub content: String,
    pub tags: Vec<&'a str>,
}

#[derive(Debug)]
pub struct ListOptions {
    pub offset: u32,
    pub limit: u32,
    pub tags: Vec<i32>,
    pub all_tags: bool,
    pub sort_by_created: bool,
    pub query: String,
}

// ========================== ERRORS ======================

#[derive(Debug, thiserror::Error, Serialize)]
pub enum ArticleError {
    #[error("Not found")]
    NotFound,
    #[error("Bad content")]
    BadContent,
    #[error("Internal")]
    Internal(
        #[source]
        #[from]
        #[serde(skip)]
        sqlx::Error,
    ),
}

// ========================== FUNCTIONS ===================

pub async fn list(
    db: &Db,
    user: i32,
    opt: ListOptions,
) -> Result<Vec<ArticlePreview>, ArticleError> {
    let (or_tags, all_tags) = if opt.all_tags {
        (vec![], opt.tags)
    } else {
        (opt.tags, vec![])
    };
    Ok(sqlx::query_as(
        "SELECT a.id, a.title, a.created_on, a.updated_on,
        ARRAY_REMOVE(ARRAY_AGG(t.name), NULL) as tags,
        CASE WHEN coalesce($7, '') = ''
            THEN 0
            ELSE ts_rank_cd(search_vector, plainto_tsquery(a.language, $7), 32)
        END AS rank,
        CASE WHEN coalesce($7, '') = ''
            THEN a.preview
            ELSE ts_headline(a.raw_text, plainto_tsquery(a.language, $7),
                'StartSel=**, StopSel=**,
                MaxWords=30, MinWords=15,
                MaxFragments=5')
        END AS preview
        FROM articles a
                LEFT JOIN article_tags at ON at.article_id = a.id
                LEFT JOIN tags t ON at.tag_id = t.id
        WHERE a.user_id = $1 
            AND CASE WHEN coalesce($7, '') = ''
            THEN TRUE
            ELSE plainto_tsquery(a.language, $7) @@ a.search_vector 
        END 
        GROUP BY a.id
	    HAVING 
		    (CARDINALITY($4::int[]) = 0 OR $4::int[] && ARRAY_AGG(t.id))
		    AND (CARDINALITY($5::int[]) = 0 OR $5::int[] <@ ARRAY_AGG(t.id))
        ORDER BY 
            rank DESC,
            CASE WHEN ($6)
                THEN a.created_on
                ELSE a.updated_on 
            END DESC
        LIMIT $2 OFFSET $3",
    )
    .bind(user)
    .bind(opt.limit)
    .bind(opt.offset)
    .bind(or_tags)
    .bind(all_tags)
    .bind(opt.sort_by_created)
    .bind(opt.query)
    .fetch_all(db)
    .await?)
}

pub async fn get(db: &Db, user: i32, id: i32) -> Result<Article, ArticleError> {
    let article = sqlx::query_as::<_, Article>(
        "
        SELECT a.id, a.title, a.content, a.created_on, a.updated_on, 
            ARRAY_REMOVE(ARRAY_AGG(t.name), NULL) as tags
        FROM articles a 
            LEFT JOIN article_tags at ON at.article_id = a.id 
            LEFT JOIN tags t ON at.tag_id = t.id 
        WHERE a.id = $1 AND a.user_id = $2
        GROUP BY a.id",
    )
    .bind(id)
    .bind(user)
    .fetch_optional(db)
    .await?;
    article.ok_or(ArticleError::NotFound)
}

pub async fn create(
    db: &Db,
    link_analyzer: &LinkAnalyzer,
    user: i32,
    article: ArticleInsert<'_>,
) -> Result<i32, ArticleError> {
    if !validate_content(&article) {
        return Err(ArticleError::BadContent);
    }
    // insert values
    let info = extractor::extract_article(&article.content);
    let id: i32 = sqlx::query_scalar(
        "
        INSERT INTO articles (user_id, title, content, raw_text, preview, language)
        VALUES ($1, $2, $3, $4, $5, CAST($6 AS regconfig))
        RETURNING id",
    )
    .bind(user)
    .bind(article.title)
    .bind(article.content)
    .bind(info.text)
    .bind(info.preview)
    .bind(info.language)
    .fetch_one(db)
    .await?;
    // update tags
    super::tags::update_article_tags(db, user, id, &article.tags).await?;
    // update links
    super::links::update_article_links(link_analyzer, id, user, &info.links).await?;
    Ok(id)
}

pub async fn update(
    db: &Db,
    link_analyzer: &LinkAnalyzer,
    user: i32,
    article: ArticleInsert<'_>,
) -> Result<(), ArticleError> {
    // check id
    let id = article.id.ok_or(ArticleError::BadContent)?;
    if !validate_content(&article) {
        return Err(ArticleError::BadContent);
    }
    // check user access
    match get_user_id(db, id).await? {
        Some(stored_user) if stored_user == user => (),
        _ => return Err(ArticleError::NotFound),
    }
    // update values
    let info = extractor::extract_article(&article.content);
    sqlx::query(
        "
        UPDATE articles SET 
        title = $2, content = $3, raw_text = $4, preview = $5, language = CAST($6 AS regconfig)
        WHERE id = $1
        ",
    )
    .bind(id)
    .bind(article.title)
    .bind(article.content)
    .bind(info.text)
    .bind(info.preview)
    .bind(info.language)
    .execute(db)
    .await?;
    // update tags
    super::tags::update_article_tags(db, user, id, &article.tags).await?;
    // update links
    super::links::update_article_links(link_analyzer, id, user, &info.links).await?;
    Ok(())
}

pub async fn delete(db: &Db, user: i32, id: i32) -> Result<(), ArticleError> {
    match get_user_id(db, id).await? {
        Some(stored_user) if stored_user == user => (),
        _ => return Err(ArticleError::NotFound),
    };
    // update links
    super::links::delete_article_links(db, id).await?;
    // update tags
    super::tags::update_article_tags(db, user, id, &[]).await?;
    // delete
    sqlx::query("DELETE FROM articles WHERE id = $1")
        .bind(id)
        .execute(db)
        .await?;
    Ok(())
}

// ========================== HELPERS =====================

async fn get_user_id(db: &Db, id: i32) -> Result<Option<i32>, sqlx::Error> {
    sqlx::query_scalar(
        "
        SELECT user_id FROM articles
        WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(db)
    .await
}

fn validate_content(
    ArticleInsert {
        title,
        content,
        tags,
        ..
    }: &ArticleInsert<'_>,
) -> bool {
    (1..).contains(&title.len())
        && (1..).contains(&content.len())
        && tags.iter().all(|tag| !tag.is_empty())
}
