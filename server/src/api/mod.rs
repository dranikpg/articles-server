use std::sync;

use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::response::Responder;
use rocket::serde::{Serialize, json::Json};
use rocket::{Request, Response, State};
use crate::security;

pub mod article;
pub mod links;
pub mod tags;
pub mod user;

// ========================= TYPES ========================

type Db = State<sync::Arc<crate::db::Db>>;
type LinkAnalyzer = State<sync::Arc<crate::db::links::Analyzer>>;
type SecurityConfig = State<sync::Arc<security::Config>>;
pub struct User {
    pub id: i32,
}

// ========================== ERRORS ======================

pub trait AsHttpStatus {
    fn status(&self) -> Status;
}

impl AsHttpStatus for sqlx::Error {
    fn status(&self) -> Status {
        Status::InternalServerError
    }
}

#[derive(Serialize)]
pub struct ErrorDesc<T: std::error::Error> {
    pub kind: T,
    pub message: String,
}

pub struct ApiErr<T>(T);

impl<T> From<T> for ApiErr<T> {
    fn from(t: T) -> Self {
        ApiErr(t)
    }
}

impl<'r, 'o: 'r, T> Responder<'r, 'o> for ApiErr<T>
where
    T: std::error::Error + AsHttpStatus + Serialize,
{
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'o> {
        let message = self.0.to_string();
        let status = self.0.status();
        // log internal errors
        if status == Status::InternalServerError {
            log::warn!("Internal error: {:?}", self.0);
        }
        let json = Json(ErrorDesc {
            kind: self.0,
            message,
        });
        Response::build_from(json.respond_to(request)?)
            .status(status)
            .ok()
    }
}

// ========================== GUARDS ======================

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = String;
    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let jwt = match request.cookies().get("jwt").map(|c| c.value()) {
            Some(jwt) => jwt,
            None => return Outcome::Failure((Status::Unauthorized, "".to_owned())),
        };
        let user = match crate::db::user::authenticate(jwt) {
            Some(id) => User { id },
            None => return Outcome::Failure((Status::Unauthorized, "".to_owned())),
        };
        Outcome::Success(user)
    }
}
