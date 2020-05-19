use chrono::NaiveDateTime;
use rusqlite::{params, Connection as RQConnection, Result as RQResult, Row};
use serde::Serialize;
use std::ops::Add;
use std::sync::Mutex;

pub struct Connection {
    conn: Mutex<RQConnection>,
}

#[derive(Debug, Serialize)]
pub struct Article {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub created: String,
}
#[derive(Debug, Serialize)]
pub struct ArticlePreview {
    id: i32,
    title: String,
}
pub struct ArticleProto<'a> {
    pub id: Option<i32>,
    pub title: &'a str,
    pub content: &'a str,
}

impl Article {
    fn from_row(r: &Row) -> RQResult<Article> {
        let created: NaiveDateTime = r.get("created_ts")?;
        Ok(Article {
            id: r.get("id")?,
            title: r.get("title")?,
            content: r.get("content")?,
            created: created.format("%e %B %y").to_string(),
        })
    }
}
impl ArticlePreview {
    fn from_row(r: &Row) -> RQResult<ArticlePreview> {
        Ok(ArticlePreview {
            id: r.get("id")?,
            title: r.get("title")?,
        })
    }
}

impl Connection {
    pub fn new() -> Self {
        let url = std::env::var("LOCATION")
            .expect("LOCATION not set")
            .add("/ass.db");
        let conn = RQConnection::open(url).expect("Failed to connect");

        conn.execute(include_str!("create.sql"), params![])
            .expect("Failed to create tables");

        Connection {
            conn: Mutex::new(conn),
        }
    }
    pub fn update(&self, proto: ArticleProto) -> RQResult<i32> {
        let c = self.conn.lock().unwrap();
        match proto.id {
            Some(id) => {
                c.execute(
                    r#"UPDATE articles SET
                                        title = ?,
                                        content = ?
                                    WHERE id = ?"#,
                    params![proto.title, proto.content, id],
                )?;
                Ok(id)
            }
            None => {
                c.execute(
                    r#"INSERT INTO articles (title, content) VALUES (?,?)"#,
                    &[proto.title, proto.content],
                )?;
                Ok(c.last_insert_rowid() as i32)
            }
        }
    }
    pub fn get(&self, id: i32) -> RQResult<Option<Article>> {
        let c = self.conn.lock().unwrap();
        let mut smt = c.prepare("SELECT * FROM articles WHERE id = ?")?;
        let mut iter = smt.query_map(params![id], Article::from_row)?;
        match iter.next() {
            Some(Ok(a)) => Ok(Some(a)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }
    pub fn get_many(&self, ids: &[i32]) -> RQResult<Vec<ArticlePreview>> {
        let c = self.conn.lock().unwrap();
        let qs = {
            let mut q = String::from("SELECT id, title FROM articles WHERE id IN (");
            for id in ids {
                q.push_str(&format!("{}", id));
                q.push(',');
            }
            q.pop();
            q.push_str(");");
            q
        };
        let mut smt = c.prepare(&qs)?;
        let iter = smt.query_map(params![], ArticlePreview::from_row)?;
        let out = iter.filter(|e| e.is_ok()).map(|e| e.unwrap()).collect();
        Ok(out)
    }

    pub fn list(&self) -> RQResult<Vec<ArticlePreview>> {
        let c = self.conn.lock().unwrap();
        let mut smt = c.prepare("SELECT id, title FROM articles ORDER BY id DESC")?;
        let iter = smt.query_map(params![], ArticlePreview::from_row)?;

        let out = iter.filter(|e| e.is_ok()).map(|e| e.unwrap()).collect();
        Ok(out)
    }
    pub fn delete(&self, id: i32) -> RQResult<()> {
        let c = self.conn.lock().unwrap();
        c.execute("DELETE FROM articles WHERE id = ?", params![id])?;
        Ok(())
    }
}
