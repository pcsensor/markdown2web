use std::{path::Path, sync::Mutex};

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use chrono::Utc;
use rand::{Rng, distributions::Alphanumeric};
use rusqlite::{Connection, OptionalExtension, params};
use serde::Serialize;

use crate::error::AppResult;

#[derive(Debug, Clone, Serialize)]
pub struct BuildEvent {
    pub level: String,
    pub message: String,
    pub created_at: String,
}

pub struct AppDatabase {
    conn: Mutex<Connection>,
}

impl AppDatabase {
    pub fn open(path: &Path) -> AppResult<Self> {
        let conn = Connection::open(path)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn initialize(&self, username: &str, password: &str) -> AppResult<()> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS admin_users (
                username TEXT PRIMARY KEY,
                password_hash TEXT NOT NULL,
                created_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS sessions (
                token TEXT PRIMARY KEY,
                username TEXT NOT NULL,
                created_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS build_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                level TEXT NOT NULL,
                message TEXT NOT NULL,
                created_at TEXT NOT NULL
            );
            "#,
        )?;

        let exists: Option<String> = conn
            .query_row(
                "SELECT username FROM admin_users WHERE username = ?1",
                params![username],
                |row| row.get(0),
            )
            .optional()?;
        if exists.is_none() {
            let hash = hash_password(password)?;
            conn.execute(
                "INSERT INTO admin_users(username, password_hash, created_at) VALUES (?1, ?2, ?3)",
                params![username, hash, Utc::now().to_rfc3339()],
            )?;
        }
        Ok(())
    }

    pub fn verify_user(&self, username: &str, password: &str) -> AppResult<bool> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let hash: Option<String> = conn
            .query_row(
                "SELECT password_hash FROM admin_users WHERE username = ?1",
                params![username],
                |row| row.get(0),
            )
            .optional()?;
        Ok(match hash {
            Some(hash) => verify_password(&hash, password),
            None => false,
        })
    }

    pub fn create_session(&self, username: &str) -> AppResult<String> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let token: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(48)
            .map(char::from)
            .collect();
        conn.execute(
            "INSERT INTO sessions(token, username, created_at) VALUES (?1, ?2, ?3)",
            params![token, username, Utc::now().to_rfc3339()],
        )?;
        Ok(token)
    }

    pub fn session_user(&self, token: &str) -> AppResult<Option<String>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let user = conn
            .query_row(
                "SELECT username FROM sessions WHERE token = ?1",
                params![token],
                |row| row.get(0),
            )
            .optional()?;
        Ok(user)
    }

    pub fn delete_session(&self, token: &str) -> AppResult<()> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        conn.execute("DELETE FROM sessions WHERE token = ?1", params![token])?;
        Ok(())
    }

    pub fn log_build(&self, level: &str, message: &str) -> AppResult<()> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        conn.execute(
            "INSERT INTO build_events(level, message, created_at) VALUES (?1, ?2, ?3)",
            params![level, message, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    pub fn recent_builds(&self, limit: usize) -> AppResult<Vec<BuildEvent>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn.prepare(
            "SELECT level, message, created_at FROM build_events ORDER BY id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(BuildEvent {
                level: row.get(0)?,
                message: row.get(1)?,
                created_at: row.get(2)?,
            })
        })?;
        Ok(rows.filter_map(Result::ok).collect())
    }
}

fn hash_password(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    Ok(Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(crate::error::AppError::internal)?
        .to_string())
}

fn verify_password(hash: &str, password: &str) -> bool {
    let parsed = match PasswordHash::new(hash) {
        Ok(parsed) => parsed,
        Err(_) => return false,
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}
