use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Individual bookmark item.
///
/// Holds the descriptive text, destination address, and visual folder categories.
///
/// # Fields
/// * `title` - Short name of the bookmarked resource.
/// * `url` - Address link of the bookmarked resource.
/// * `folder` - Categorization folder label.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FavoriteItem {
    /// Friendly bookmark title.
    pub title: String,
    /// Destination web or offline URL.
    pub url: String,
    /// Categorization folder for visual grouping.
    pub folder: String,
}

/// Serialized collection container wrapping a list of [FavoriteItem] nodes.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct FavoritesList {
    /// Dynamic collection of bookmarks.
    pub items: Vec<FavoriteItem>,
}

/// Service managing persistent actions, loading, saving, and grouping bookmarks.
///
/// # Fields
/// * `list` - Dynamic RAM-cached container for bookmarks.
/// * `file_path` - Location of persistent storage file (typically `favorites.json`).
pub struct FavoritesManager {
    /// In-memory collection list.
    pub list: FavoritesList,
    /// File path to save/load JSON records.
    pub file_path: String,
}

impl FavoritesManager {
    /// Initializes a new manager, loading records from specified path.
    ///
    /// If the path does not exist, defaults are populated and persisted automatically.
    ///
    /// # Arguments
    ///
    /// * `file_path` - The target JSON storage path.
    ///
    /// # Returns
    ///
    /// Returns an operational [FavoritesManager].
    ///
    /// # Examples
    ///
    /// ```
    /// use isearch_cli::browser::favorites::FavoritesManager;
    /// let mgr = FavoritesManager::new("temp_favs.json");
    /// assert!(mgr.list.items.len() >= 3);
    /// # std::fs::remove_file("temp_favs.json").unwrap();
    /// ```
    pub fn new(file_path: &str) -> Self {
        let mut mgr = Self {
            list: FavoritesList::default(),
            file_path: file_path.to_string(),
        };
        let _ = mgr.load();
        mgr
    }

    /// Loads favorites records from local storage.
    ///
    /// Falls back to seeding initial standard default items if the JSON file is missing.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or `Err(String)` containing the disk/parsing warning.
    ///
    /// # Errors
    ///
    /// Returns an error if file permissions prevent reads or if contents are malformed JSON.
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

    /// Persists current list of bookmarks as JSON structure to disk.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on successful write, or an error description.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory is unwritable or disk space is exhausted.
    pub fn save(&self) -> Result<(), String> {
        let path = Path::new(&self.file_path);
        let data = serde_json::to_string_pretty(&self.list).map_err(|e| e.to_string())?;
        fs::write(path, data).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Adds a new bookmark, clean folder categories, and immediately saves to disk.
    ///
    /// # Arguments
    ///
    /// * `title` - Label for the item.
    /// * `url` - Hyperlink target.
    /// * `folder` - Grouping folder. If blank, defaults to "General".
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success.
    ///
    /// # Errors
    ///
    /// Returns an error if the disk persistence write fails.
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

    /// Deletes any bookmarks containing the matching URL and updates storage.
    ///
    /// # Arguments
    ///
    /// * `url` - Target URL string slice to prune.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization/write fails.
    pub fn remove(&mut self, url: &str) -> Result<(), String> {
        self.list.items.retain(|item| item.url != url);
        self.save()
    }

    /// Prunes a bookmark item located at a specific index.
    ///
    /// # Arguments
    ///
    /// * `index` - Position inside the dynamic list to remove.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())`.
    ///
    /// # Errors
    ///
    /// Returns an error if index was in bounds but disk serialization failed.
    pub fn remove_at(&mut self, index: usize) -> Result<(), String> {
        if index < self.list.items.len() {
            self.list.items.remove(index);
            self.save()?;
        }
        Ok(())
    }

    /// Searches active title and URL buffers case-insensitively.
    ///
    /// # Arguments
    ///
    /// * `query` - Search criteria string slice.
    ///
    /// # Returns
    ///
    /// Returns a vector of matching clones [FavoriteItem].
    pub fn search(&self, query: &str) -> Vec<FavoriteItem> {
        let q = query.to_lowercase();
        self.list
            .items
            .iter()
            .filter(|item| {
                item.title.to_lowercase().contains(&q) || item.url.to_lowercase().contains(&q)
            })
            .cloned()
            .collect()
    }

    /// Lists all unique folder categories currently tracked across bookmarks.
    ///
    /// # Returns
    ///
    /// Returns a sorted list of unique folder strings starting with "All" and "General".
    pub fn folders(&self) -> Vec<String> {
        let mut f = vec!["All".to_string(), "General".to_string()];
        for item in &self.list.items {
            if !f.contains(&item.folder) {
                f.push(item.folder.clone());
            }
        }
        f
    }

    /// Exports bookmarks JSON representation to an external file.
    ///
    /// # Arguments
    ///
    /// * `path_str` - Path target to save exported configuration.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or `Err(String)` on write failures.
    ///
    /// # Errors
    ///
    /// Returns an error if permissions prevent file creation or writing.
    pub fn export_to_file(&self, path_str: &str) -> Result<(), String> {
        let data = serde_json::to_string_pretty(&self.list).map_err(|e| e.to_string())?;
        fs::write(path_str, data).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Imports external bookmark lists, skipping duplicate URLs, and saves changes.
    ///
    /// # Arguments
    ///
    /// * `path_str` - Path string reference to read JSON structure from.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if successful, or an error.
    ///
    /// # Errors
    ///
    /// Returns an error if the target file is missing, unreadable, or contains invalid bookmarks schemas.
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
