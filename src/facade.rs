use crate::data::{ArticlePreview, ArticleProto, Connection};
use rocket::request::Form;
use rocket::response::Redirect;
use rocket::State;

use rocket_contrib::templates::Template;

use serde::Serialize;

use crate::textindex::TextIndex;

fn render_base(v: Vec<ArticlePreview>, curq: &str) -> Template {
    #[derive(Serialize)]
    struct Data {
        items: Vec<ArticlePreview>,
        curq: String,
    };
    let items = v;
    let dt = Data {
        items,
        curq: curq.to_string(),
    };
    Template::render("main", &dt)
}

#[get("/")]
pub fn list(db: State<Connection>) -> Option<Template> {
    let v = db.list().ok()?;
    Some(render_base(v, ""))
}

#[get("/?<q>")]
pub fn search(q: String, db: State<Connection>, tv: State<TextIndex>) -> Option<Template> {
    let ids = tv.search(&q)?;
    let v = if ids.is_empty() {
        Vec::new()
    } else {
        db.get_many(&ids).ok()?
    };
    Some(render_base(v, &q))
}

#[get("/<id>")]
pub fn get(id: i32, db: State<Connection>) -> Option<Template> {
    let dt = match db.get(id) {
        Ok(Some(a)) => a,
        _ => return None,
    };
    Some(Template::render("view", dt))
}

#[get("/update?<id>")]
pub fn update(id: Option<i32>, db: State<Connection>) -> Option<Template> {
    #[derive(Serialize, Debug, Default)]
    struct Data {
        title: String,
        content: String,
    }
    let dt = match id {
        Some(x) => match db.get(x) {
            Ok(Some(article)) => Data {
                title: article.title,
                content: article.content,
            },
            _ => return None,
        },
        None => Data::default(),
    };
    Some(Template::render("edit", &dt))
}

#[derive(FromForm)]
pub struct ArticleForm {
    title: String,
    content: String,
}
#[post("/update?<id>", data = "<form>")]
pub fn update_post(
    id: Option<i32>,
    form: Form<ArticleForm>,
    db: State<Connection>,
    tv: State<TextIndex>,
) -> Redirect {
    let proto = ArticleProto {
        id,
        title: &form.title,
        content: &form.content,
    };
    let id = db.update(proto);
    match id {
        Ok(x) => {
            match db.get(x) {
                Ok(Some(art)) => {
                    tv.delete(art.id);
                    tv.insert(art);
                }
                _ => {}
            }
            Redirect::to(uri!(get: x))
        }
        _ => Redirect::to(uri!(list)),
    }
}

#[post("/delete?<id>")]
pub fn delete(id: i32, db: State<Connection>, tv: State<TextIndex>) -> Redirect {
    let _res = db.delete(id);
    tv.delete(id);
    Redirect::to(uri!(list))
}
