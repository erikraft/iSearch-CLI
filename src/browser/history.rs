use serde::{Serialize, Deserialize};
use rusqlite::{params, Connection};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct HistoryItem {
    pub id: Option<i64>,
    pub title: String,
    pub url: String,
    pub visited_at: String,
    pub visit_count: i64,
}

pub struct HistoryManager {
    pub db_path: String,
}

impl HistoryManager {
    pub fn new(db_path: &str) -> Self {
        let mgr = Self {
            db_path: db_path.to_string(),
        };
        let _ = mgr.init_db();
        mgr
    }

    fn connect(&self) -> Result<Connection, String> {
        Connection::open(&self.db_path).map_err(|e| e.to_string())
    }

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
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

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
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn delete_selected(&self, id: i64) -> Result<(), String> {
        let conn = self.connect()?;
        conn.execute("DELETE FROM history WHERE id = ?1", params![id]).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn delete_all(&self) -> Result<(), String> {
        let conn = self.connect()?;
        conn.execute("DELETE FROM history", []).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_all(&self, search_query: &str, filter_domain: &str, sort_by: &str) -> Result<Vec<HistoryItem>, String> {
        let conn = self.connect()?;
        let mut sql = "SELECT id, title, url, visited_at, visit_count FROM history WHERE 1=1".to_string();
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
            1 => stmt.query(params![params_vec[0]]).map_err(|e| e.to_string())?,
            2 => stmt.query(params![params_vec[0], params_vec[1]]).map_err(|e| e.to_string())?,
            3 => stmt.query(params![params_vec[0], params_vec[1], params_vec[2]]).map_err(|e| e.to_string())?,
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

    pub fn export_to_file(&self, path_str: &str) -> Result<(), String> {
        let items = self.get_all("", "", "date")?;
        let data = serde_json::to_string_pretty(&items).map_err(|e| e.to_string())?;
        fs::write(path_str, data).map_err(|e| e.to_string())?;
        Ok(())
    }

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

pub fn group_by_domain(items: &[HistoryItem]) -> Vec<(String, Vec<(usize, HistoryItem)>)> {
    let mut groups: Vec<(String, Vec<(usize, HistoryItem)>)> = Vec::new();
    for (original_idx, item) in items.iter().enumerate() {
        let domain = extract_domain(&item.url);
        let domain_name = if domain.is_empty() { "Local/Other".to_string() } else { domain };
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
        mgr.add_visit("Rust Lang", "https://rust-lang.org/index.html").unwrap();

        // Get all
        let list = mgr.get_all("", "", "date").unwrap();
        assert_eq!(list.len(), 2);

        // Search
        let search_res = mgr.get_all("Google", "", "date").unwrap();
        assert_eq!(search_res.len(), 1);
        assert_eq!(search_res[0].title, "Google");

        // Domain extraction
        assert_eq!(extract_domain("https://www.google.com/search?q=rust"), "www.google.com");

        // Cleanup
        let _ = fs::remove_file(temp_db);
    }
}
