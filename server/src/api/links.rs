use rocket::serde::json::Json;
use super::{ApiErr, AsHttpStatus, Db, User};
use crate::db::links;

// ========================== TYPES =======================

type ApiResult<T> = Result<Json<T>, ApiErr<links::LinkError>>;

// ========================== ERRORS ======================

impl AsHttpStatus for links::LinkError {
    fn status(&self) -> rocket::http::Status {
        use links::LinkError;
        use rocket::http::Status;
        match &self {
            LinkError::Internal(_) => Status::InternalServerError,
        }
    }
}

// ========================= RESPONDERS ===================

#[get("/links")]
pub async fn list(db: &Db, user: User) -> ApiResult<Vec<links::Link>> {
    Ok(Json(links::list(db.as_ref(), user.id, (0, 100)).await?))
}
