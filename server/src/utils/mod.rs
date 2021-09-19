pub mod extractor;

pub fn detect_language(content: &str) -> &'static str {
    match whatlang::detect_lang(&content) {
        Some(whatlang::Lang::Eng) => "english",
        Some(whatlang::Lang::Deu) => "german",
        Some(whatlang::Lang::Rus) => "russian",
        _ => "english",
    }
}
