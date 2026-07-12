use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};

pub struct HistoryService {
    conn: Connection,
}

impl HistoryService {
    pub fn new() -> Result<Self, HistoryError> {
        let db_path = dirs::data_dir()
            .ok_or(HistoryError::NoDataDir)?
            .join("frog/history.db");

        std::fs::create_dir_all(db_path.parent().unwrap())?;

        let conn = Connection::open(db_path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS history (
                id INTEGER PRIMARY KEY,
                text TEXT NOT NULL,
                language TEXT,
                confidence REAL,
                image_path TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                tags TEXT
            )",
            [],
        )?;

        // Full-text search index
        conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS history_fts USING fts5(
                text,
                content='history',
                content_rowid='id'
            )",
            [],
        )?;

        Ok(Self { conn })
    }

    pub fn add(&self, item: &HistoryItem) -> Result<i64, HistoryError> {
        self.conn.execute(
            "INSERT INTO history (text, language, confidence, image_path, tags)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                item.text,
                item.language,
                item.confidence,
                item.image_path,
                item.tags
            ],
        )?;

        let id = self.conn.last_insert_rowid();

        // Update FTS index
        self.conn.execute(
            "INSERT INTO history_fts (rowid, text) VALUES (?1, ?2)",
            params![id, item.text],
        )?;

        Ok(id)
    }

    pub fn search(&self, query: &str) -> Result<Vec<HistoryItem>, HistoryError> {
        let mut stmt = self.conn.prepare(
            "SELECT h.* FROM history h
             JOIN history_fts fts ON h.id = fts.rowid
             WHERE history_fts MATCH ?1
             ORDER BY rank",
        )?;

        let items = stmt
            .query_map([query], |row| {
                Ok(HistoryItem {
                    id: row.get(0)?,
                    text: row.get(1)?,
                    language: row.get(2)?,
                    confidence: row.get(3)?,
                    image_path: row.get(4)?,
                    created_at: row.get(5)?,
                    tags: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(items)
    }
}
