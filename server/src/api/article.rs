use rocket::serde::json::Json;
use super::{ApiErr, AsHttpStatus, Db, LinkAnalyzer, User};
use crate::db::article;

// ========================== TYPES =======================

type ApiResult<T> = Result<Json<T>, ApiErr<article::ArticleError>>;

// ========================== ERRORS ======================

impl AsHttpStatus for article::ArticleError {
    fn status(&self) -> rocket::http::Status {
        use article::ArticleError;
        use rocket::http::Status;
        match &self {
            ArticleError::BadContent => Status::BadRequest,
            ArticleError::NotFound => Status::NotFound,
            ArticleError::Internal(_) => Status::InternalServerError,
        }
    }
}

// ========================= RESPONDERS ===================

#[get("/article/<id>")]
pub async fn get(db: &Db, id: i32, user: User) -> ApiResult<article::Article> {
    Ok(Json(article::get(db.as_ref(), user.id, id).await?))
}

#[get("/article?<from>&<limit>&<tags>&<sort_by>&<all_tags>&<query>")]
pub async fn list(
    db: &Db,
    user: User,
    from: Option<u32>,
    limit: Option<u32>,
    tags: Option<String>,
    all_tags: Option<bool>,
    sort_by: Option<&'_ str>,
    query: Option<String>,
) -> ApiResult<Vec<article::ArticlePreview>> {
    let tags = tags.map(|s| s.split(',').map(|s| s.parse()).flatten().collect());
    let options = article::ListOptions {
        offset: from.unwrap_or(0),
        limit: limit.unwrap_or(10),
        tags: tags.unwrap_or(vec![]),
        all_tags: all_tags.unwrap_or(false),
        sort_by_created: sort_by.map(|s| s == "created").unwrap_or(false),
        query: query.unwrap_or_default(),
    };
    Ok(Json(article::list(db.as_ref(), user.id, options).await?))
}

#[post("/article", data = "<article>")]
pub async fn update(
    db: &Db,
    link_analyzer: &LinkAnalyzer,
    user: User,
    article: Json<article::ArticleInsert<'_>>,
) -> ApiResult<i32> {
    let id = if let Some(id) = article.id {
        article::update(db.as_ref(), link_analyzer.as_ref(), user.id, article.0).await?;
        id
    } else {
        article::create(db.as_ref(), link_analyzer.as_ref(), user.id, article.0).await?
    };
    Ok(Json(id))
}

#[delete("/article/<id>")]
pub async fn delete(db: &Db, user: User, id: i32) -> ApiResult<()> {
    article::delete(db.as_ref(), user.id, id).await?;
    Ok(Json(()))
}
