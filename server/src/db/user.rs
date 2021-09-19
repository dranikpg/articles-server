use super::Db;
use rocket::serde::Serialize;

// ========================== TYPES =======================

pub type UserId = i32;
pub type UserToken = String;

#[derive(Debug, sqlx::FromRow)]
struct UserAuth {
    id: UserId,
    password: String,
}

// ========================== ERRORS ======================

#[derive(thiserror::Error, Debug, Serialize)]
pub enum AuthError {
    #[error("Email already exists")]
    EmailExists,
    #[error("Improper crendentials")]
    ImproperCredentials,
    #[error("Wrong credentials")]
    WrongCredentials,
    #[error("Forbidden")]
    Forbidden, 
    #[error("Internal error")]
    Internal(
        #[source]
        #[from]
        #[serde(skip)]
        sqlx::Error,
    ),
}

// ========================== FUNCTIONS ===================

pub async fn register(db: &Db, email: &str, password: &str) -> Result<UserToken, AuthError> {
    if !(3..60).contains(&email.len()) || (5..12).contains(&password.len()) {
        return Err(AuthError::ImproperCredentials);
    }
    if sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)")
        .bind(email).fetch_one(db).await? {
        return Err(AuthError::EmailExists);
    }
    let hashed_password = bcrypt::hash(password, 5).expect("Bcrypt error");
    let id: i32 = sqlx::query_scalar(
        "INSERT INTO users (email, password) VALUES ($1, $2) RETURNING id")
        .bind(email)
        .bind(hashed_password)
        .fetch_one(db)
        .await?;
    Ok(generate_user_token(id, email))
}

pub async fn login(db: &Db, email: &str, given_password: &str) -> Result<UserToken, AuthError> {
    let user_q = sqlx::query_as::<_, UserAuth>(
        "SELECT id, password FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(db)
        .await?;
    if let Some(UserAuth { id, password }) = user_q {
        if bcrypt::verify(given_password, &password).expect("Bcrypt error") {
            Ok(generate_user_token(id, email))
        } else {
            Err(AuthError::WrongCredentials)
        }
    } else {
        Err(AuthError::WrongCredentials)
    }
}

pub fn authenticate(token: &str) -> Option<UserId> {
    verify_token(token)
}

// ========================== HELPER ======================

const BASE_SECRET: &'static [u8] = std::include_bytes!("../../.secret.key");
const_assert!(BASE_SECRET.len() >= 32);

const fn generate_hash_secret(i: usize) -> u8 {
    BASE_SECRET[i]
}
const HASH_SECRET: [u8; 32] = array_const_fn_init::array_const_fn_init![generate_hash_secret; 32];

fn generate_token_hash(id: UserId, email: &str) -> String {
    let info = format!("{};{}", id, email);
    blake3::keyed_hash(&HASH_SECRET, info.as_bytes()).to_string()
}

fn generate_user_token(id: UserId, email: &str) -> String {
    let hash = generate_token_hash(id, email);
    base64::encode(format!("{};{};{}", id, email, hash))
}

fn verify_token(token_b64: &str) -> Option<UserId> {
    let token_raw = base64::decode(token_b64).ok()?;
    let token = std::str::from_utf8(&token_raw).ok()?;
    let mut parts = token.split(";");
    let (id, email, hash) = (parts.next()?, parts.next()?, parts.next()?);
    let id = id.parse().ok()?;
    if generate_token_hash(id, email) == hash {
        Some(id)
    } else {
        None
    }
}
