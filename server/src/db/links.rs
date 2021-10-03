use super::Db;
use rocket::serde::{Deserialize, Serialize};
use rocket::tokio::{
    self,
    sync::{mpsc, oneshot},
};
use std::{collections::HashSet, sync::Arc};
use thirtyfour::{error::WebDriverError, prelude::*};

// ========================== TYPES =======================

enum Message {
    ID(i32),
    Shutdown(oneshot::Sender<()>),
}

pub struct Analyzer {
    db: Arc<Db>,
    process: std::process::Child,
    sender: mpsc::Sender<Message>,
}

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct Link {
    pub id: i32,
    pub article_id: i32,
    pub article_title: String,
    pub url: String,
    pub title: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
struct LinkUrl {
    id: i32,
    url: String,
}

struct WebInfo {
    title: String,
    content: String,
    language: &'static str,
    screenshot: Vec<u8>,
}

#[derive(Debug, thiserror::Error, Serialize)]
pub enum LinkError {
    #[error("Internal")]
    Internal(
        #[from]
        #[source]
        #[serde(skip)]
        sqlx::Error,
    ),
}

#[derive(Debug, Deserialize)]
pub struct AnalyzerConfig {
    pub url: String,
}

impl Analyzer {
    pub async fn send(&self, id: i32) {
        self.sender.send(Message::ID(id)).await.ok();
    }
}

// ========================== FUNCTIONS ===================

pub async fn list(db: &Db, user: i32, (offset, limit): (u32, u32)) -> Result<Vec<Link>, LinkError> {
    Ok(sqlx::query_as(
        "
        SELECT l.id, l.article_id, l.url, l.title,
        (SELECT title FROM articles a WHERE a.id = l.article_id) as article_title
        FROM links l
        WHERE l.user_id = $1
        ORDER BY l.id DESC
        LIMIT $2 OFFSET $3",
    )
    .bind(user)
    .bind(limit)
    .bind(offset)
    .fetch_all(db)
    .await?)
}

pub async fn update_article_links(
    analyzer: &Analyzer,
    article: i32,
    user: i32,
    links: &[impl AsRef<str>],
) -> Result<(), sqlx::Error> {
    let db = analyzer.db.as_ref();
    let current_links: Vec<String> = sqlx::query_as::<_, LinkUrl>(
        "SELECT id, url FROM links WHERE article_id = $1")
        .bind(article).fetch_all(db)
        .await?
        .into_iter().map(|l| l.url).collect();

    let links_hs = links.iter().map(|s| s.as_ref()).collect::<HashSet<&str>>();
    let current_links_hs = current_links
        .iter()
        .map(|s| s.as_ref())
        .collect::<HashSet<&str>>();

    // TODO: improve
    for link in current_links_hs.difference(&links_hs) {
        sqlx::query("DELETE FROM links WHERE article_id = $1 AND url = $2")
            .bind(article).bind(link)
            .execute(db)
            .await?;
    }

    // TODO: improve + replace with async streams
    let mut new_links = false;
    for link in links_hs.difference(&current_links_hs) {
        sqlx::query("INSERT INTO links (article_id, user_id, url)
            VALUES ($1, $2, $3)")
            .bind(article).bind(user).bind(link)
            .execute(db).await?;
        new_links = true;
    }

    // send links to analyzer
    // TODO: dont fail quietly
    if new_links {
        analyzer.send(article).await;
    }

    Ok(())
}

pub async fn delete_article_links(db: &Db, article: i32) 
-> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM links WHERE article_id = $1")
        .bind(article)
        .execute(db).await?;
    Ok(())
}

pub async fn start_analyzer(
    db: Arc<Db>,
    config: AnalyzerConfig,
) -> Result<Analyzer, WebDriverError> {
    log::info!("Starting link analyzer");
    let (sx, rx) = tokio::sync::mpsc::channel::<Message>(100);
    let process = std::process::Command::new("./chromedriver")
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to start chromedriver");

    log::info!("Waiting for chromedriver to start...");
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let driver = {
        let mut caps = DesiredCapabilities::chrome();
        caps.add_chrome_arg("headless");
        caps.add_chrome_arg("--no-sandbox");
        WebDriver::new(&config.url, &caps).await?
    };
    let dbc = db.clone();
    tokio::spawn(async move {
        analyze_links(rx, dbc, driver)
            .await
            .expect("Link analyzer failed");
    });
    Ok(Analyzer {
        sender: sx,
        process,
        db,
    })
}

pub async fn stop_analyzer(analyzer: Arc<Analyzer>) {
    let (sx, rx) = oneshot::channel::<()>();
    analyzer.sender.send(Message::Shutdown(sx)).await.ok();
    rx.await.ok();
    Arc::try_unwrap(analyzer)
        .map(|mut analyzer| analyzer.process.kill())
        .ok();
}

pub async fn analyze_fresh(analyzer: &Analyzer) -> Result<(), sqlx::Error> {
    let ids = get_fresh_links(analyzer.db.as_ref()).await?;
    log::info!("Sending {} fresh links to analyzer", ids.len());
    for id in ids {
        analyzer.send(id).await;
    }
    Ok(())
}

async fn analyze_links(
    mut recv: mpsc::Receiver<Message>,
    db: Arc<Db>,
    driver: WebDriver,
) -> Result<(), Box<dyn std::error::Error>> {
    let shutdown_sx = loop {
        let article = match recv.recv().await {
            Some(Message::ID(article)) => article,
            Some(Message::Shutdown(sx)) => break Some(sx),
            _ => break None,
        };
        let links = match get_link_urls(&db, article).await {
            Ok(links) => links,
            _ => continue,
        };
        log::trace!("LINK {:?}", links);
        for link in links {
            let info = match analyze_web_content(&driver, &link.url).await {
                Ok(info) => info,
                _ => continue,
            };
            sqlx::query(
                "UPDATE links SET fresh = FALSE, title = $2, content = $3, 
                language = CAST($4 AS regconfig),
                search_vector = to_tsvector(language, title || ' ' || content),
                screenshot = $5
                WHERE id = $1")
            .bind(link.id)
            .bind(info.title)
            .bind(info.content)
            .bind(info.language)
            .bind(info.screenshot)
            .execute(db.as_ref())
            .await
            .ok();
        }
    };
    log::debug!("Closing webdriver");
    // driver.quit().await?;
    if let Some(sx) = shutdown_sx {
        sx.send(()).ok();
    }
    Ok(())
}

async fn analyze_web_content(driver: &WebDriver, url: &str) -> WebDriverResult<WebInfo> {
    log::trace!("Analyzing link {}", url);
    driver.get(url).await?;
    let title = driver.title().await?;
    let root = driver.find_element(By::Tag("body")).await?;
    let screenshot = root.screenshot_as_png().await?;
    let content = root.text().await?;
    let language = crate::utils::detect_language(&content);
    Ok(WebInfo {
        title,
        content,
        language,
        screenshot,
    })
}

async fn get_link_urls(db: &Db, article: i32) -> Result<Vec<LinkUrl>, sqlx::Error> {
    Ok(sqlx::query_as::<_, LinkUrl>(
        "SELECT id, url FROM links WHERE article_id = $1 AND fresh = TRUE ")
    .bind(article)
    .fetch_all(db)
    .await?)
}

async fn get_fresh_links(db: &Db) -> Result<Vec<i32>, sqlx::Error> {
    Ok(sqlx::query_scalar(
        "SELECT DISTINCT(article_id) FROM links WHERE fresh = TRUE"
    )
    .fetch_all(db)
    .await?)
}
