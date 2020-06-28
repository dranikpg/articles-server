#![feature(proc_macro_hygiene, decl_macro)]

extern crate dotenv;
extern crate env_logger;
#[macro_use]
extern crate log;

#[macro_use]
extern crate rocket;

extern crate rusqlite;

#[macro_use]
extern crate tantivy;


extern crate pulldown_cmark;

mod data;
mod facade;
mod textindex;

use crate::data::Connection;
use crate::textindex::TextIndex;
use rocket_contrib::templates::Template;

fn main() {
    dotenv::dotenv().ok();
    env_logger::init();

    if std::env::var("LOCATION").is_err() {
        std::env::set_var("LOCATION", "/home/vlad/articles");
    }

    rocket::ignite()
        .manage(Connection::new())
        .manage(TextIndex::new())
        .attach(Template::fairing())
        .mount(
            "/",
            routes![
                facade::list,
                facade::search,
                facade::get,
                facade::update,
                facade::update_post,
                facade::delete
            ],
        )
        .launch();
}
