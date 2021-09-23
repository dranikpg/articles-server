use std::sync::Arc;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate ormx;
#[macro_use]
extern crate static_assertions;
#[macro_use]
extern crate const_format;

mod api;
mod db;
mod utils;
pub mod security;

// ========================== INIT ========================

async fn run() {
    env_logger::init();

    let rocket = rocket::build();
    let figment = rocket.figment();

    // Init db
    let db = {
        let config: db::Config = figment
            .extract_inner("db")
            .expect("No valid db config found");
        let db = db::init(config).await.expect("Failed to init DB");
        Arc::new(db)
    };

    // Init link analyzer
    let link_analyzer = {
        let config: db::links::AnalyzerConfig = figment
            .extract_inner("selenium")
            .expect("No valid selenium config found");
        let analyzer = db::links::start_analyzer(db.clone(), config)
            .await
            .expect("Failed to start link analyzer");
        db::links::analyze_fresh(&analyzer)
            .await
            .expect("Failed to send fresh links to analyzer");
        Arc::new(analyzer)
    };

    let security = {
        let config: security::Config = figment.extract_inner("security")
            .expect("Failed to parse security");
        Arc::new(config)
    };

    // register state
    let rocket = rocket
        .manage(db)
        .manage(link_analyzer.clone())
        .manage(security);

    // register routes
    let rocket = rocket.mount(
        "/",
        routes![
            api::user::info,
            api::user::login,
            api::article::list,
            api::article::get,
            api::article::update,
            api::article::delete,
            api::tags::list,
            api::links::list
        ],
    );

    // launch
    rocket.launch().await.expect("Rocket crashed");

    // shutdown

    log::debug!("rocket shutdown");

    db::links::stop_analyzer(link_analyzer).await;
}

fn main() {
    let rt = rocket::tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to build tokio runtime");
    rt.block_on(async {
        run().await;
    });
}
