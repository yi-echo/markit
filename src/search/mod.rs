use crate::models::Snippet;

pub mod fuzzy;

pub trait Searcher {
    fn search(&self, query: &str, snippets: &[Snippet]) -> Vec<ScoredSnippet>;
}

#[derive(Debug, Clone)]
pub struct ScoredSnippet {
    pub snippet: Snippet,
    pub score: u32,
    pub matched_fields: Vec<MatchedField>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatchedField {
    Name,
    Description,
    Content,
    Tag(String),
}

impl ScoredSnippet {
    pub fn new(snippet: Snippet, score: u32, matched_fields: Vec<MatchedField>) -> Self {
        Self {
            snippet,
            score,
            matched_fields,
        }
    }
}

