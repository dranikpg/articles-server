use rocket::serde::json::Json;
use super::{ApiErr, AsHttpStatus, Db, User};
use crate::db::tags;

// ========================== TYPES =======================

type ApiResult<T> = Result<Json<T>, ApiErr<tags::TagError>>;

// ========================== ERRORS ======================

impl AsHttpStatus for tags::TagError {
    fn status(&self) -> rocket::http::Status {
        use rocket::http::Status;
        use tags::TagError;
        match &self {
            TagError::Internal(_) => Status::InternalServerError,
        }
    }
}

// ========================= RESPONDERS ===================

#[get("/tags")]
pub async fn list(db: &Db, user: User) -> ApiResult<Vec<tags::Tag>> {
    Ok(Json(tags::list(db.as_ref(), user.id).await?))
}
