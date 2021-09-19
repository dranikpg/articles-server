use rocket::serde::Deserialize;
use sqlx::ConnectOptions;

pub mod article;
pub mod links;
pub mod tags;
pub mod user;
// ========================== INIT ========================

pub type Db = sqlx::PgPool;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub url: String,
    pub database: String,
    pub user: String,
    pub password: String,
    pub connections: u8,
}

pub async fn init(config: Config) -> Result<Db, sqlx::Error> {
    let Config {
        url,
        database,
        user,
        password,
        connections,
    } = config;
    let mut conn_options = sqlx::postgres::PgConnectOptions::new()
        .username(&user)
        .password(&password)
        .host(&url)
        .database(&database)
        .application_name("articles");
    conn_options
        .log_statements(log::LevelFilter::Trace)
        .log_slow_statements(
            log::LevelFilter::Debug,
            std::time::Duration::from_millis(10),
        );
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(connections as u32)
        .connect_with(conn_options)
        .await?;

    log::info!("Running migrations");
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
