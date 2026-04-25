use rusqlite::Connection;
use std::sync::Arc;
use parking_lot::Mutex;
use std::path::Path;

#[derive(Clone)]
pub struct PendingNotesDb {
    pub conn: Arc<Mutex<Connection>>,
}

impl PendingNotesDb {
    pub fn new(db_path: impl AsRef<Path>) -> Self {
        let conn = Connection::open(db_path).expect("Failed to open database");
        conn.execute(
            "CREATE TABLE IF NOT EXISTS pending_notes (
                id INTEGER PRIMARY KEY,
                term TEXT NOT NULL,
                reading TEXT NOT NULL,
                definition TEXT NOT NULL,
                tags TEXT NOT NULL,
                serialized_entry TEXT NOT NULL
            )",
            [],
        ).expect("Failed to create table");
        
        Self {
            conn: Arc::new(Mutex::new(conn)),
        }
    }

    pub fn insert_note(&self, term: &str, reading: &str, definition: &str, tags: &str, serialized_entry: &str) {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO pending_notes (term, reading, definition, tags, serialized_entry) VALUES (?1, ?2, ?3, ?4, ?5)",
            [term, reading, definition, tags, serialized_entry],
        ).expect("Failed to insert note");
    }

    pub fn get_all_notes(&self) -> Vec<(i32, String, String, String, String, String)> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare("SELECT id, term, reading, definition, tags, serialized_entry FROM pending_notes").unwrap();
        let rows = stmt.query_map([], |row: &rusqlite::Row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?))
        }).unwrap();
        
        rows.map(|r: rusqlite::Result<(i32, String, String, String, String, String)>| r.unwrap()).collect()
    }

    pub fn delete_note(&self, id: i32) {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM pending_notes WHERE id = ?1", [id]).expect("Failed to delete note");
    }
}
