use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use rusqlite::{params, Connection};
use serde_json::json;
use voteme_api::Vote;

pub struct Database {
    path: PathBuf,
    conn: Mutex<Connection>,
}

impl Database {
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, String> {
        let path = path.into();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create db directory {parent:?}: {e}"))?;
            }
        }

        let conn = Connection::open(&path)
            .map_err(|e| format!("Failed to open sqlite db at {path:?}: {e}"))?;

        let db = Self {
            path,
            conn: Mutex::new(conn),
        };
        db.init()?;
        Ok(db)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn init(&self) -> Result<(), String> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| "Database mutex poisoned".to_string())?;

        conn.execute_batch(
            "
            PRAGMA foreign_keys = ON;
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;

            CREATE TABLE IF NOT EXISTS votes (
                id            INTEGER PRIMARY KEY AUTOINCREMENT,
                service_name   TEXT NOT NULL,
                username       TEXT NOT NULL,
                address        TEXT,
                vote_timestamp TEXT,
                received_at_ms INTEGER NOT NULL,
                vote_json      TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_votes_username ON votes(username);
            CREATE INDEX IF NOT EXISTS idx_votes_service ON votes(service_name);
            CREATE INDEX IF NOT EXISTS idx_votes_received ON votes(received_at_ms);

            CREATE UNIQUE INDEX IF NOT EXISTS uniq_votes_natural
            ON votes(service_name, username, IFNULL(address, ''), IFNULL(vote_timestamp, ''));
            ",
        )
        .map_err(|e| format!("Failed to initialize sqlite schema: {e}"))?;

        Ok(())
    }

    pub fn insert_vote(&self, vote: &Vote) -> Result<(), String> {
        let received_at_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;

        let vote_json = json!({
            "service_name": vote.service_name,
            "username": vote.username,
            "address": vote.address,
            "timestamp": vote.timestamp,
        })
        .to_string();

        let conn = self
            .conn
            .lock()
            .map_err(|_| "Database mutex poisoned".to_string())?;

        conn.execute(
            "
            INSERT INTO votes(
                service_name,
                username,
                address,
                vote_timestamp,
                received_at_ms,
                vote_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(service_name, username, IFNULL(address, ''), IFNULL(vote_timestamp, '')) DO UPDATE SET
                received_at_ms = excluded.received_at_ms,
                vote_json = excluded.vote_json
            ",
            params![
                vote.service_name,
                vote.username,
                vote.address,
                vote.timestamp,
                received_at_ms,
                vote_json
            ],
        )
        .map(|_| ())
        .map_err(|e| format!("Failed to insert vote: {e}"))
    }
}
