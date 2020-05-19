use std::sync::Mutex;
use tantivy::directory::MmapDirectory;
use tantivy::query::QueryParser;
use tantivy::schema::{Field, Schema, INDEXED, STORED, TEXT};
use tantivy::{Index, IndexReader, IndexWriter, ReloadPolicy, Term};

use crate::data::Article;
use std::cell::RefCell;
use tantivy::collector::TopDocs;

struct TextIndexInner {
    index: Index,
    index_writer: IndexWriter,
    reader: IndexReader,
    id_fd: Field,
    title_fd: Field,
    content_fd: Field,
}
impl TextIndexInner {
    fn new() -> Self {
        let (index, schema) = {
            let path = std::env::var("LOCATION").expect("LOCATION not set");
            let path_dir = MmapDirectory::open(path).expect("Failed to create tantivy directory");

            let schema = {
                let mut schema_builder = Schema::builder();
                schema_builder.add_i64_field("id", INDEXED | STORED);
                schema_builder.add_text_field("title", TEXT);
                schema_builder.add_text_field("content", TEXT);
                schema_builder.build()
            };

            (
                Index::open_or_create(path_dir, schema.clone()).expect("Failed to create index"),
                schema,
            )
        };

        let index_writer = index
            .writer_with_num_threads(1, 50_000_000)
            .expect("Failed to create writer");

        let id_fd = schema.get_field("id").unwrap();
        let title_fd = schema.get_field("title").unwrap();
        let content_fd = schema.get_field("content").unwrap();

        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommit)
            .num_searchers(1)
            .try_into()
            .expect("Failed to build reader");

        TextIndexInner {
            index,
            index_writer,
            reader,
            id_fd,
            title_fd,
            content_fd,
        }
    }
}

pub struct TextIndex {
    inner: Mutex<RefCell<TextIndexInner>>,
}
impl TextIndex {
    pub fn new() -> Self {
        let inner = TextIndexInner::new();
        TextIndex {
            inner: Mutex::new(RefCell::new(inner)),
        }
    }
    pub fn delete(&self, id: i32) {
        let tv_g = self.inner.lock().unwrap();
        let mut tv = tv_g.borrow_mut();
        let term = Term::from_field_i64(tv.id_fd, id as i64);
        tv.index_writer.delete_term(term);
        tv.index_writer.commit().ok();
    }
    pub fn insert(&self, a: Article) {
        let tv_g = self.inner.lock().unwrap();
        let mut tv = tv_g.borrow_mut();
        let doc = doc!(
            tv.id_fd => a.id as i64,
            tv.title_fd => a.title,
            tv.content_fd => a.content
        );
        tv.index_writer.add_document(doc);
        tv.index_writer.commit().ok();
    }
    pub fn search(&self, query: &str) -> Option<Vec<i32>> {
        let tv_g = self.inner.lock().unwrap();
        let tv = tv_g.borrow();
        let searcher = tv.reader.searcher();
        let query_parser = QueryParser::for_index(&tv.index, vec![tv.title_fd, tv.content_fd]);
        let query = query_parser.parse_query(query).unwrap();
        let top_docs = searcher.search(&query, &TopDocs::with_limit(20)).unwrap();

        let mut out = Vec::with_capacity(20);
        for (_score, doc_adress) in top_docs {
            let doc = searcher.doc(doc_adress).unwrap();
            out.push(doc.get_first(tv.id_fd)?.i64_value() as i32);
        }
        Some(out)
    }
}
