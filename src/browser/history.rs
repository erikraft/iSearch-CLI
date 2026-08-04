use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Represents an individual recorded web navigation element.
///
/// # Fields
/// * `id` - Optional SQLite autoincremented unique row identifier.
/// * `title` - Cached web page title.
/// * `url` - Destination address link.
/// * `visited_at` - Timestamp of the most recent visit.
/// * `visit_count` - Accumulative times the URL has been visited.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct HistoryItem {
    /// SQLite primary key representation.
    pub id: Option<i64>,
    /// Header/Title of the visited URL.
    pub title: String,
    /// Destination address link.
    pub url: String,
    /// Standard local database timestamp string.
    pub visited_at: String,
    /// Frequency of visits to this specific URL.
    pub visit_count: i64,
}

/// Service managing SQL operations on the persistent history database.
///
/// # Fields
/// * `db_path` - Disk path of the local SQLite `.db` file.
pub struct HistoryManager {
    /// local SQLite file path.
    pub db_path: String,
}

impl HistoryManager {
    /// Instantiates a new history manager and initializes the backing table schema if it does not exist.
    ///
    /// # Arguments
    ///
    /// * `db_path` - File path target for the SQLite database.
    ///
    /// # Returns
    ///
    /// Returns an initialized [HistoryManager].
    ///
    /// # Examples
    ///
    /// ```
    /// use isearch_cli::browser::history::HistoryManager;
    /// let mgr = HistoryManager::new("temp_history.db");
    /// # std::fs::remove_file("temp_history.db").unwrap();
    /// ```
    pub fn new(db_path: &str) -> Self {
        let mgr = Self {
            db_path: db_path.to_string(),
        };
        let _ = mgr.init_db();
        mgr
    }

    /// Spawns a connection to the SQLite database.
    ///
    /// # Returns
    ///
    /// Returns standard [Connection] on success, or a String explanation if connection fails.
    ///
    /// # Errors
    ///
    /// Returns an error if the database file is locked or cannot be created.
    fn connect(&self) -> Result<Connection, String> {
        Connection::open(&self.db_path).map_err(|e| e.to_string())
    }

    /// Formulates the primary history table schema if it has not been configured yet.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success.
    ///
    /// # Errors
    ///
    /// Returns an error if schema execution fails.
    pub fn init_db(&self) -> Result<(), String> {
        let conn = self.connect()?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                url TEXT NOT NULL UNIQUE,
                visited_at TEXT NOT NULL,
                visit_count INTEGER DEFAULT 1
            )",
            [],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Log a newly visited URL or updates its count/timestamp if already present.
    ///
    /// # Arguments
    ///
    /// * `title` - Descriptive name of the visited resource.
    /// * `url` - Destination address link.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to database fails.
    pub fn add_visit(&self, title: &str, url: &str) -> Result<(), String> {
        let conn = self.connect()?;
        conn.execute(
            "INSERT INTO history (title, url, visited_at, visit_count)
             VALUES (?1, ?2, datetime('now', 'localtime'), 1)
             ON CONFLICT(url) DO UPDATE SET
                title = excluded.title,
                visited_at = datetime('now', 'localtime'),
                visit_count = visit_count + 1",
            params![title.trim(), url.trim()],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Deletes a specific history record matching the autoincremented ID.
    ///
    /// # Arguments
    ///
    /// * `id` - SQLite table primary key representing the target row.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success.
    ///
    /// # Errors
    ///
    /// Returns an error if SQL execution fails.
    pub fn delete_selected(&self, id: i64) -> Result<(), String> {
        let conn = self.connect()?;
        conn.execute("DELETE FROM history WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Truncates the entire history table, prunining all navigation rows.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success.
    ///
    /// # Errors
    ///
    /// Returns an error if SQL execution fails.
    pub fn delete_all(&self) -> Result<(), String> {
        let conn = self.connect()?;
        conn.execute("DELETE FROM history", [])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_all(
        &self,
        search_query: &str,
        filter_domain: &str,
        sort_by: &str,
    ) -> Result<Vec<HistoryItem>, String> {
        let conn = self.connect()?;
        let mut sql =
            "SELECT id, title, url, visited_at, visit_count FROM history WHERE 1=1".to_string();
        let mut params_vec: Vec<String> = Vec::new();

        if !search_query.trim().is_empty() {
            sql.push_str(" AND (title LIKE ? OR url LIKE ?)");
            params_vec.push(format!("%{}%", search_query.trim()));
            params_vec.push(format!("%{}%", search_query.trim()));
        }

        if !filter_domain.trim().is_empty() {
            sql.push_str(" AND url LIKE ?");
            params_vec.push(format!("%{}%", filter_domain.trim()));
        }

        if sort_by == "visits" {
            sql.push_str(" ORDER BY visit_count DESC, visited_at DESC");
        } else {
            sql.push_str(" ORDER BY visited_at DESC");
        }

        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;

        let mut rows = match params_vec.len() {
            0 => stmt.query([]).map_err(|e| e.to_string())?,
            1 => stmt
                .query(params![params_vec[0]])
                .map_err(|e| e.to_string())?,
            2 => stmt
                .query(params![params_vec[0], params_vec[1]])
                .map_err(|e| e.to_string())?,
            3 => stmt
                .query(params![params_vec[0], params_vec[1], params_vec[2]])
                .map_err(|e| e.to_string())?,
            _ => stmt.query([]).map_err(|e| e.to_string())?,
        };

        let mut items = Vec::new();
        while let Some(row) = rows.next().map_err(|e| e.to_string())? {
            items.push(HistoryItem {
                id: Some(row.get(0).map_err(|e| e.to_string())?),
                title: row.get(1).map_err(|e| e.to_string())?,
                url: row.get(2).map_err(|e| e.to_string())?,
                visited_at: row.get(3).map_err(|e| e.to_string())?,
                visit_count: row.get(4).map_err(|e| e.to_string())?,
            });
        }
        Ok(items)
    }

    /// Exports standard history entries as structured JSON data into an external path.
    ///
    /// # Arguments
    ///
    /// * `path_str` - System path string slice where JSON content will be saved.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success.
    ///
    /// # Errors
    ///
    /// Returns an error if the path cannot be opened or written to.
    pub fn export_to_file(&self, path_str: &str) -> Result<(), String> {
        let items = self.get_all("", "", "date")?;
        let data = serde_json::to_string_pretty(&items).map_err(|e| e.to_string())?;
        fs::write(path_str, data).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Imports external history items from a JSON representation and UPSERTs them into SQL tables.
    ///
    /// # Arguments
    ///
    /// * `path_str` - Path location containing history JSON schemas.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success.
    ///
    /// # Errors
    ///
    /// Returns an error if the file does not exist, cannot be read, or is malformed JSON.
    pub fn import_from_file(&self, path_str: &str) -> Result<(), String> {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err("File does not exist".to_string());
        }
        let data = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let imported: Vec<HistoryItem> = serde_json::from_str(&data).map_err(|e| e.to_string())?;
        let conn = self.connect()?;
        for item in imported {
            let _ = conn.execute(
                "INSERT INTO history (title, url, visited_at, visit_count)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(url) DO UPDATE SET
                    title = excluded.title,
                    visited_at = excluded.visited_at,
                    visit_count = MAX(visit_count, excluded.visit_count)",
                params![item.title, item.url, item.visited_at, item.visit_count],
            );
        }
        Ok(())
    }
}

/// Sanitizes a full URL string to extract its core web host domain.
///
/// Strips typical protocol headers like `https://` or `http://`, port offsets, and trailing sub-paths.
///
/// # Arguments
///
/// * `url` - Full input URL address.
///
/// # Returns
///
/// Returns the parsed host domain name as a String.
///
/// # Examples
///
/// ```
/// use isearch_cli::browser::history::extract_domain;
/// let dom = extract_domain("https://example.org:8080/index");
/// assert_eq!(dom, "example.org");
/// ```
pub fn extract_domain(url: &str) -> String {
    let mut clean = url.trim().to_lowercase();
    if clean.starts_with("https://") {
        clean = clean["https://".len()..].to_string();
    } else if clean.starts_with("http://") {
        clean = clean["http://".len()..].to_string();
    }
    if let Some(slash_idx) = clean.find('/') {
        clean = clean[..slash_idx].to_string();
    }
    if let Some(colon_idx) = clean.find(':') {
        clean = clean[..colon_idx].to_string();
    }
    clean
}

/// Structurally aggregates history items based on their formatted YYYY-MM-DD local timestamps.
///
/// Each date group binds a list of references with their corresponding index positions.
///
/// # Arguments
///
/// * `items` - Reference slice of [HistoryItem] objects to cluster.
///
/// # Returns
///
/// Returns grouped arrays pairing dates to original indices and clones.
pub fn group_by_date(items: &[HistoryItem]) -> Vec<(String, Vec<(usize, HistoryItem)>)> {
    let mut groups: Vec<(String, Vec<(usize, HistoryItem)>)> = Vec::new();
    for (original_idx, item) in items.iter().enumerate() {
        let date = if item.visited_at.len() >= 10 {
            item.visited_at[..10].to_string()
        } else {
            "Unknown Date".to_string()
        };
        if let Some(group) = groups.iter_mut().find(|(d, _)| d == &date) {
            group.1.push((original_idx, item.clone()));
        } else {
            groups.push((date, vec![(original_idx, item.clone())]));
        }
    }
    groups
}

/// Structurally aggregates history items based on their domain names.
///
/// # Arguments
///
/// * `items` - Reference slice of [HistoryItem] objects to cluster.
///
/// # Returns
///
/// Returns grouped arrays pairing extracted domains to original indices and clones.
pub fn group_by_domain(items: &[HistoryItem]) -> Vec<(String, Vec<(usize, HistoryItem)>)> {
    let mut groups: Vec<(String, Vec<(usize, HistoryItem)>)> = Vec::new();
    for (original_idx, item) in items.iter().enumerate() {
        let domain = extract_domain(&item.url);
        let domain_name = if domain.is_empty() {
            "Local/Other".to_string()
        } else {
            domain
        };
        if let Some(group) = groups.iter_mut().find(|(d, _)| d == &domain_name) {
            group.1.push((original_idx, item.clone()));
        } else {
            groups.push((domain_name, vec![(original_idx, item.clone())]));
        }
    }
    groups
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_management() {
        let temp_db = "temp_history_test.db";
        let _ = fs::remove_file(temp_db);

        let mgr = HistoryManager::new(temp_db);

        // Add visit
        mgr.add_visit("Google", "https://google.com").unwrap();
        mgr.add_visit("Rust Lang", "https://rust-lang.org/index.html")
            .unwrap();

        // Get all
        let list = mgr.get_all("", "", "date").unwrap();
        assert_eq!(list.len(), 2);

        // Search
        let search_res = mgr.get_all("Google", "", "date").unwrap();
        assert_eq!(search_res.len(), 1);
        assert_eq!(search_res[0].title, "Google");

        // Domain extraction
        assert_eq!(
            extract_domain("https://www.google.com/search?q=rust"),
            "www.google.com"
        );

        // Cleanup
        let _ = fs::remove_file(temp_db);
    }
}
