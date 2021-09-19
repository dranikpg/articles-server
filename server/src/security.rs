use rocket::serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub allow_register: bool
}