use comrak::{
    nodes::{AstNode, NodeLink, NodeValue},
    parse_document, Arena, ComrakOptions,
};
use std::cell::RefCell;

const MAX_PREVIEW_LEN: usize = 140;

pub struct Info {
    pub preview: String,
    pub text: String,
    pub links: Vec<String>,
    pub language: &'static str,
}

pub fn extract_article(content: &str) -> Info {
    let arena = Arena::new();
    let root = parse_document(&arena, content, &ComrakOptions::default());

    let text = RefCell::new(Vec::new());
    let links = RefCell::new(Vec::new());
    iter_nodes(root, &|node| match &mut node.data.borrow_mut().value {
        NodeValue::Text(ref entry) => {
            if let Ok(entry) = String::from_utf8(entry.to_owned()) {
                text.borrow_mut().push(entry);
            }
        }
        NodeValue::Link(NodeLink { ref url, .. }) => {
            if let Ok(link) = String::from_utf8(url.to_owned()) {
                links.borrow_mut().push(link);
            }
        }
        _ => (),
    });
    let preview = text
        .borrow()
        .iter()
        .map(|s| s.split_whitespace())
        .flatten()
        .fold(String::new(), |mut preview, part| {
            if preview.len() + part.len() <= MAX_PREVIEW_LEN {
                preview.push_str(part);
                preview.push(' ');
            }
            preview
        });
    let text = text.take().join(" ");
    let language = super::detect_language(&text);
    Info {
        preview,
        text,
        links: links.take(),
        language,
    }
}

fn iter_nodes<'a, F>(node: &'a AstNode<'a>, f: &F)
where
    F: Fn(&'a AstNode<'a>),
{
    f(node);
    for c in node.children() {
        iter_nodes(c, f);
    }
}
