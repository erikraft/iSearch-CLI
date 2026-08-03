use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FavoriteItem {
    pub title: String,
    pub url: String,
    pub folder: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct FavoritesList {
    pub items: Vec<FavoriteItem>,
}

pub struct FavoritesManager {
    pub list: FavoritesList,
    pub file_path: String,
}

impl FavoritesManager {
    pub fn new(file_path: &str) -> Self {
        let mut mgr = Self {
            list: FavoritesList::default(),
            file_path: file_path.to_string(),
        };
        let _ = mgr.load();
        mgr
    }

    pub fn load(&mut self) -> Result<(), String> {
        let path = Path::new(&self.file_path);
        if path.exists() {
            let data = fs::read_to_string(path).map_err(|e| e.to_string())?;
            self.list = serde_json::from_str(&data).map_err(|e| e.to_string())?;
        } else {
            // Default bookmarks
            self.list.items = vec![
                FavoriteItem {
                    title: "Rust Language".to_string(),
                    url: "https://www.rust-lang.org".to_string(),
                    folder: "Programming".to_string(),
                },
                FavoriteItem {
                    title: "Hacker News".to_string(),
                    url: "https://news.ycombinator.com".to_string(),
                    folder: "News".to_string(),
                },
                FavoriteItem {
                    title: "Google Search".to_string(),
                    url: "https://www.google.com".to_string(),
                    folder: "General".to_string(),
                },
            ];
            let _ = self.save();
        }
        Ok(())
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Path::new(&self.file_path);
        let data = serde_json::to_string_pretty(&self.list).map_err(|e| e.to_string())?;
        fs::write(path, data).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn add(&mut self, title: &str, url: &str, folder: &str) -> Result<(), String> {
        let clean_folder = if folder.trim().is_empty() {
            "General".to_string()
        } else {
            folder.trim().to_string()
        };
        self.list.items.push(FavoriteItem {
            title: title.trim().to_string(),
            url: url.trim().to_string(),
            folder: clean_folder,
        });
        self.save()
    }

    pub fn remove(&mut self, url: &str) -> Result<(), String> {
        self.list.items.retain(|item| item.url != url);
        self.save()
    }

    pub fn remove_at(&mut self, index: usize) -> Result<(), String> {
        if index < self.list.items.len() {
            self.list.items.remove(index);
            self.save()?;
        }
        Ok(())
    }

    pub fn search(&self, query: &str) -> Vec<FavoriteItem> {
        let q = query.to_lowercase();
        self.list.items.iter()
            .filter(|item| item.title.to_lowercase().contains(&q) || item.url.to_lowercase().contains(&q))
            .cloned()
            .collect()
    }

    pub fn folders(&self) -> Vec<String> {
        let mut f = vec!["All".to_string(), "General".to_string()];
        for item in &self.list.items {
            if !f.contains(&item.folder) {
                f.push(item.folder.clone());
            }
        }
        f
    }

    pub fn export_to_file(&self, path_str: &str) -> Result<(), String> {
        let data = serde_json::to_string_pretty(&self.list).map_err(|e| e.to_string())?;
        fs::write(path_str, data).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn import_from_file(&mut self, path_str: &str) -> Result<(), String> {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err("File does not exist".to_string());
        }
        let data = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let imported: FavoritesList = serde_json::from_str(&data).map_err(|e| e.to_string())?;
        for item in imported.items {
            if !self.list.items.iter().any(|i| i.url == item.url) {
                self.list.items.push(item);
            }
        }
        self.save()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_favorites_management() {
        let temp_file = "temp_favorites_test.json";
        let _ = fs::remove_file(temp_file);

        let mut mgr = FavoritesManager::new(temp_file);
        // Initially should have defaults
        assert_eq!(mgr.list.items.len(), 3);

        // Add
        mgr.add("My Test Title", "https://test.com", "Dev").unwrap();
        assert_eq!(mgr.list.items.len(), 4);

        // Search
        let search_res = mgr.search("test");
        assert_eq!(search_res.len(), 1);
        assert_eq!(search_res[0].title, "My Test Title");

        // Folders
        let folders = mgr.folders();
        assert!(folders.contains(&"Dev".to_string()));

        // Remove
        mgr.remove("https://test.com").unwrap();
        assert_eq!(mgr.list.items.len(), 3);

        let _ = fs::remove_file(temp_file);
    }
}
