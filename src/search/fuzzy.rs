use nucleo_matcher::{Matcher, Utf32String};

use crate::models::Snippet;
use crate::search::{MatchedField, ScoredSnippet, Searcher};

pub struct FuzzySearcher {
    matcher: Matcher,
}

impl FuzzySearcher {
    pub fn new() -> Self {
        let matcher = Matcher::new(nucleo_matcher::Config::DEFAULT);
        Self { matcher }
    }

    fn match_text(&mut self, query: &str, text: &str) -> Option<u32> {
        if text.is_empty() || query.is_empty() {
            return None;
        }

        let query_utf32 = Utf32String::from(query);
        let text_utf32 = Utf32String::from(text);

        self.matcher
            .fuzzy_match(text_utf32.slice(..), query_utf32.slice(..))
            .map(|score| score as u32)
    }

    fn search_in_snippet(&mut self, query: &str, snippet: &Snippet) -> Option<ScoredSnippet> {
        let mut total_score = 0u32;
        let mut matched_fields = Vec::new();
        let mut has_match = false;

        if let Some(score) = self.match_text(query, &snippet.name) {
            total_score += score * 4;
            matched_fields.push(MatchedField::Name);
            has_match = true;
        }

        if let Some(score) = self.match_text(query, &snippet.description) {
            total_score += score * 2;
            matched_fields.push(MatchedField::Description);
            has_match = true;
        }

        if let Some(score) = self.match_text(query, &snippet.content) {
            total_score += score;
            matched_fields.push(MatchedField::Content);
            has_match = true;
        }

        for tag in &snippet.tags {
            if let Some(score) = self.match_text(query, tag) {
                total_score += score * 3;
                matched_fields.push(MatchedField::Tag(tag.clone()));
                has_match = true;
            }
        }

        if has_match {
            Some(ScoredSnippet::new(snippet.clone(), total_score, matched_fields))
        } else {
            None
        }
    }
}

impl Default for FuzzySearcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Searcher for FuzzySearcher {
    fn search(&self, query: &str, snippets: &[Snippet]) -> Vec<ScoredSnippet> {
        let mut searcher = FuzzySearcher::new();
        let mut results: Vec<ScoredSnippet> = snippets
            .iter()
            .filter_map(|snippet| searcher.search_in_snippet(query, snippet))
            .collect();

        results.sort_by(|a, b| b.score.cmp(&a.score));
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_snippet(name: &str, description: &str, content: &str, tags: Vec<&str>) -> Snippet {
        Snippet {
            name: name.to_string(),
            description: description.to_string(),
            content: content.to_string(),
            executable: true,
            tags: tags.iter().map(|s| s.to_string()).collect(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_search_by_name() {
        let snippets = vec![
            create_test_snippet("docker-clean", "Clean docker", "docker system prune", vec!["docker"]),
            create_test_snippet("git-commit", "Git commit", "git commit -m", vec!["git"]),
        ];

        let searcher = FuzzySearcher::new();
        let results = searcher.search("docker", &snippets);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].snippet.name, "docker-clean");
        assert!(results[0].matched_fields.contains(&MatchedField::Name));
    }

    #[test]
    fn test_search_by_tag() {
        let snippets = vec![
            create_test_snippet("cmd1", "Desc1", "content1", vec!["docker", "cleanup"]),
            create_test_snippet("cmd2", "Desc2", "content2", vec!["git"]),
        ];

        let searcher = FuzzySearcher::new();
        let results = searcher.search("docker", &snippets);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].snippet.name, "cmd1");
        assert!(results[0]
            .matched_fields
            .iter()
            .any(|f| matches!(f, MatchedField::Tag(t) if t == "docker")));
    }

    #[test]
    fn test_search_multiple_matches() {
        let snippets = vec![
            create_test_snippet("docker-clean", "Clean docker", "docker system prune", vec!["docker"]),
            create_test_snippet("docker-build", "Build docker", "docker build", vec!["docker"]),
            create_test_snippet("git-commit", "Git commit", "git commit", vec!["git"]),
        ];

        let searcher = FuzzySearcher::new();
        let results = searcher.search("docker", &snippets);

        assert_eq!(results.len(), 2);
        assert!(results[0].score >= results[1].score);
    }

    #[test]
    fn test_search_no_matches() {
        let snippets = vec![create_test_snippet("test", "test", "test", vec!["test"])];

        let searcher = FuzzySearcher::new();
        let results = searcher.search("nonexistent", &snippets);

        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_search_empty_query() {
        let snippets = vec![create_test_snippet("test", "test", "test", vec![])];

        let searcher = FuzzySearcher::new();
        let results = searcher.search("", &snippets);

        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_search_case_insensitive() {
        let snippets = vec![create_test_snippet("Docker-Clean", "Clean", "docker prune", vec![])];

        let searcher = FuzzySearcher::new();
        let results = searcher.search("docker", &snippets);

        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_partial_match() {
        let snippets = vec![create_test_snippet("docker-cleanup-system", "Clean", "prune", vec![])];

        let searcher = FuzzySearcher::new();
        let results = searcher.search("clean", &snippets);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].snippet.name, "docker-cleanup-system");
    }
}

