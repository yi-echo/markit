use crate::{
    storage::{Storage, filter::Filter, filter::apply_filter},
    ui::TableUI,
};

pub fn search_command(storage: &dyn Storage, table_ui: &mut dyn TableUI, query: String) {
    let store = match storage.load() {
        Ok(s) => s,
        Err(_) => {
            println!("📭 No snippets saved yet.");
            return;
        }
    };

    if query.trim().is_empty() {
        println!("⚠️ Search query cannot be empty.");
        return;
    }

    let snippets = apply_filter(&store, Filter::FuzzySearch(query.clone()));

    if snippets.is_empty() {
        println!("📭 No snippets found matching: '{}'.", query);
    } else {
        println!("🔍 Found {} snippet(s) matching '{}':\n", snippets.len(), query);
        let table = table_ui.with_snippet_list(snippets);
        println!("{table}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        models::{Snippet, SnippetStore},
        storage::{Storage, StorageError},
        ui::TableUI,
    };
    use chrono::Utc;
    use std::cell::RefCell;
    use std::rc::Rc;

    struct MockStorage {
        store: SnippetStore,
        should_fail: bool,
    }

    impl Storage for MockStorage {
        fn load(&self) -> Result<SnippetStore, StorageError> {
            if self.should_fail {
                Err(StorageError::Io(std::io::Error::other("Load failed")))
            } else {
                Ok(self.store.clone())
            }
        }

        fn save(&self, _: Snippet) -> Result<(), StorageError> {
            Ok(())
        }

        fn save_all(&self, _: &SnippetStore) -> Result<(), StorageError> {
            Ok(())
        }

        fn get_backups(&self) -> Result<Vec<std::path::PathBuf>, StorageError> {
            Ok(vec![])
        }

        fn restore_backup(&self, _: &std::path::Path) -> Result<(), StorageError> {
            Ok(())
        }
    }

    struct MockTableUI {
        printed_table: Rc<RefCell<bool>>,
    }

    impl TableUI for MockTableUI {
        fn with_snippet_list(&mut self, _: Vec<Snippet>) -> comfy_table::Table {
            *self.printed_table.borrow_mut() = true;
            comfy_table::Table::new()
        }
    }

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
    fn test_search_command_success() {
        let store = SnippetStore {
            snippets: vec![
                create_test_snippet("docker-clean", "Clean docker", "docker system prune", vec!["docker"]),
                create_test_snippet("git-commit", "Git commit", "git commit -m", vec!["git"]),
            ],
        };

        let storage = MockStorage {
            store,
            should_fail: false,
        };

        let mut table_ui = MockTableUI {
            printed_table: Rc::new(RefCell::new(false)),
        };

        search_command(&storage, &mut table_ui, "docker".to_string());
        assert!(*table_ui.printed_table.borrow());
    }

    #[test]
    fn test_search_command_no_results() {
        let store = SnippetStore {
            snippets: vec![create_test_snippet("test", "test", "test", vec!["test"])],
        };

        let storage = MockStorage {
            store,
            should_fail: false,
        };

        let mut table_ui = MockTableUI {
            printed_table: Rc::new(RefCell::new(false)),
        };

        search_command(&storage, &mut table_ui, "nonexistent".to_string());
        assert!(!*table_ui.printed_table.borrow());
    }

    #[test]
    fn test_search_command_empty_query() {
        let store = SnippetStore {
            snippets: vec![create_test_snippet("test", "test", "test", vec![])],
        };

        let storage = MockStorage {
            store,
            should_fail: false,
        };

        let mut table_ui = MockTableUI {
            printed_table: Rc::new(RefCell::new(false)),
        };

        search_command(&storage, &mut table_ui, "   ".to_string());
        assert!(!*table_ui.printed_table.borrow());
    }

    #[test]
    fn test_search_command_load_failure() {
        let storage = MockStorage {
            store: SnippetStore { snippets: vec![] },
            should_fail: true,
        };

        let mut table_ui = MockTableUI {
            printed_table: Rc::new(RefCell::new(false)),
        };

        search_command(&storage, &mut table_ui, "test".to_string());
        assert!(!*table_ui.printed_table.borrow());
    }

    #[test]
    fn test_search_command_no_snippets() {
        let storage = MockStorage {
            store: SnippetStore { snippets: vec![] },
            should_fail: false,
        };

        let mut table_ui = MockTableUI {
            printed_table: Rc::new(RefCell::new(false)),
        };

        search_command(&storage, &mut table_ui, "test".to_string());
        assert!(!*table_ui.printed_table.borrow());
    }
}

