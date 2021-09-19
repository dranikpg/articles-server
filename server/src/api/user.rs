use rocket::{
    http::{Cookie, CookieJar},
    serde::{json::Json, Deserialize},
};
use super::{ApiErr, AsHttpStatus, Db, SecurityConfig, User};
use crate::db::user;

// ========================== TYPES =======================

#[derive(Deserialize)]
pub struct LoginInfo<'r> {
    pub email: &'r str,
    pub password: &'r str,
}

// ========================== ERRORS ======================

impl AsHttpStatus for user::AuthError {
    fn status(&self) -> rocket::http::Status {
        use rocket::http::Status;
        use user::AuthError;
        match &self {
            AuthError::WrongCredentials => Status::Forbidden,
            AuthError::Forbidden => Status::Forbidden,
            AuthError::EmailExists => Status::BadRequest,
            AuthError::ImproperCredentials => Status::BadRequest,
            AuthError::Internal(_) => Status::InternalServerError,
        }
    }
}

// ========================= RESPONDERS ===================

#[get("/user")]
pub async fn info(u: User) -> String {
    format!("{}", u.id)
}

#[post("/login?<register>", data = "<info>")]
pub async fn login(
    db: &Db,
    security: &SecurityConfig, 
    cookies: &CookieJar<'_>,
    register: Option<bool>,
    info: Json<LoginInfo<'_>>,
) -> Result<(), ApiErr<user::AuthError>> {
    let jwt = if register.unwrap_or(false) {
        if security.allow_register {
            user::register(db.as_ref(), info.email, info.password).await?
        } else {
            return Err(user::AuthError::Forbidden.into());
        }
    } else {
        user::login(db.as_ref(), info.email, info.password).await?
    };
    store_token(cookies, jwt);
    Ok(())
}

// ========================== HELPERS =====================

pub fn store_token(cookies: &CookieJar<'_>, token: String) {
    cookies.add(Cookie::new("jwt".to_owned(), token.to_owned()));
}
