use std::{path::Path, sync::Mutex};

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use chrono::Utc;
use rand::{Rng, distributions::Alphanumeric};
use rusqlite::{Connection, OptionalExtension, params};
use serde::Serialize;

use crate::{
    error::{AppError, AppResult},
    time,
};

#[derive(Debug, Clone, Serialize)]
pub struct BuildEvent {
    pub level: String,
    pub message: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct NoteAnnotation {
    pub id: i64,
    pub username: String,
    pub note_slug: String,
    pub start_offset: usize,
    pub end_offset: usize,
    pub quote: String,
    pub color: Option<String>,
    pub comment: Option<String>,
    pub visibility: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VideoDanmaku {
    pub id: i64,
    pub username: String,
    pub note_slug: String,
    pub video_src: String,
    pub time_ms: i64,
    pub body: String,
    pub color: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ManagedPublicUser {
    pub username: String,
    pub route_key: String,
    pub created_at: String,
    pub session_count: usize,
    pub annotation_count: usize,
}

pub struct NewAnnotation<'a> {
    pub username: &'a str,
    pub note_slug: &'a str,
    pub start_offset: usize,
    pub end_offset: usize,
    pub quote: &'a str,
    pub color: Option<&'a str>,
    pub comment: Option<&'a str>,
    pub visibility: &'a str,
}

pub struct NewVideoDanmaku<'a> {
    pub username: &'a str,
    pub note_slug: &'a str,
    pub video_src: &'a str,
    pub time_ms: i64,
    pub body: &'a str,
    pub color: &'a str,
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
            CREATE TABLE IF NOT EXISTS users (
                username TEXT PRIMARY KEY,
                password_hash TEXT NOT NULL,
                created_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS user_sessions (
                token TEXT PRIMARY KEY,
                username TEXT NOT NULL,
                created_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS annotations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL,
                note_slug TEXT NOT NULL,
                start_offset INTEGER NOT NULL,
                end_offset INTEGER NOT NULL,
                quote TEXT NOT NULL,
                color TEXT,
                comment TEXT,
                visibility TEXT NOT NULL DEFAULT 'private',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE(username, note_slug, start_offset, end_offset)
            );
            CREATE TABLE IF NOT EXISTS video_danmaku (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL,
                note_slug TEXT NOT NULL,
                video_src TEXT NOT NULL,
                time_ms INTEGER NOT NULL,
                body TEXT NOT NULL,
                color TEXT NOT NULL DEFAULT '#ffffff',
                created_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_video_danmaku_lookup
                ON video_danmaku(note_slug, video_src, time_ms, id);
            "#,
        )?;
        ensure_annotations_visibility_column(&conn)?;
        ensure_video_danmaku_color_column(&conn)?;

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

    pub fn register_public_user(&self, username: &str, password: &str) -> AppResult<bool> {
        self.create_public_user(username, password)
    }

    pub fn create_public_user(&self, username: &str, password: &str) -> AppResult<bool> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let exists: Option<String> = conn
            .query_row(
                "SELECT username FROM users WHERE username = ?1",
                params![username],
                |row| row.get(0),
            )
            .optional()?;
        if exists.is_some() {
            return Ok(false);
        }
        let hash = hash_password(password)?;
        conn.execute(
            "INSERT INTO users(username, password_hash, created_at) VALUES (?1, ?2, ?3)",
            params![username, hash, Utc::now().to_rfc3339()],
        )?;
        Ok(true)
    }

    pub fn public_user_count(&self) -> AppResult<usize> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let count = conn.query_row("SELECT COUNT(*) FROM users", [], |row| row.get::<_, i64>(0))?;
        Ok(count as usize)
    }

    pub fn list_public_users(&self) -> AppResult<Vec<ManagedPublicUser>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn.prepare(
            r#"
            SELECT
                u.username,
                u.created_at,
                (SELECT COUNT(*) FROM user_sessions s WHERE s.username = u.username) AS session_count,
                (SELECT COUNT(*) FROM annotations a WHERE a.username = u.username) AS annotation_count
            FROM users u
            ORDER BY u.created_at DESC, u.username ASC
            "#,
        )?;
        let rows = stmt.query_map([], managed_public_user_from_row)?;
        Ok(rows.filter_map(Result::ok).collect())
    }

    pub fn public_user_summary(&self, username: &str) -> AppResult<Option<ManagedPublicUser>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        conn.query_row(
            r#"
            SELECT
                u.username,
                u.created_at,
                (SELECT COUNT(*) FROM user_sessions s WHERE s.username = u.username) AS session_count,
                (SELECT COUNT(*) FROM annotations a WHERE a.username = u.username) AS annotation_count
            FROM users u
            WHERE u.username = ?1
            "#,
            params![username],
            managed_public_user_from_row,
        )
        .optional()
        .map_err(Into::into)
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

    pub fn verify_public_user(&self, username: &str, password: &str) -> AppResult<bool> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let hash: Option<String> = conn
            .query_row(
                "SELECT password_hash FROM users WHERE username = ?1",
                params![username],
                |row| row.get(0),
            )
            .optional()?;
        Ok(match hash {
            Some(hash) => verify_password(&hash, password),
            None => false,
        })
    }

    pub fn update_password(&self, username: &str, password: &str) -> AppResult<bool> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let hash = hash_password(password)?;
        let updated = conn.execute(
            "UPDATE admin_users SET password_hash = ?1 WHERE username = ?2",
            params![hash, username],
        )?;
        Ok(updated > 0)
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

    pub fn create_public_session(&self, username: &str) -> AppResult<String> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let token: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(48)
            .map(char::from)
            .collect();
        conn.execute(
            "INSERT INTO user_sessions(token, username, created_at) VALUES (?1, ?2, ?3)",
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

    pub fn public_session_user(&self, token: &str) -> AppResult<Option<String>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let user = conn
            .query_row(
                "SELECT username FROM user_sessions WHERE token = ?1",
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

    pub fn delete_public_session(&self, token: &str) -> AppResult<()> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        conn.execute("DELETE FROM user_sessions WHERE token = ?1", params![token])?;
        Ok(())
    }

    pub fn delete_sessions_for_user(&self, username: &str) -> AppResult<()> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        conn.execute(
            "DELETE FROM sessions WHERE username = ?1",
            params![username],
        )?;
        Ok(())
    }

    pub fn delete_public_sessions_for_user(&self, username: &str) -> AppResult<()> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        conn.execute(
            "DELETE FROM user_sessions WHERE username = ?1",
            params![username],
        )?;
        Ok(())
    }

    pub fn update_public_user(
        &self,
        current_username: &str,
        next_username: &str,
        next_password: Option<&str>,
    ) -> AppResult<Option<String>> {
        let mut conn = self.conn.lock().expect("db mutex poisoned");
        let current_hash: Option<String> = conn
            .query_row(
                "SELECT password_hash FROM users WHERE username = ?1",
                params![current_username],
                |row| row.get(0),
            )
            .optional()?;
        let Some(current_hash) = current_hash else {
            return Ok(None);
        };

        if current_username != next_username {
            let taken: Option<String> = conn
                .query_row(
                    "SELECT username FROM users WHERE username = ?1",
                    params![next_username],
                    |row| row.get(0),
                )
                .optional()?;
            if taken.is_some() {
                return Err(AppError::BadRequest("该用户名已被注册。".into()));
            }
        }

        let password_hash = match next_password {
            Some(password) => hash_password(password)?,
            None => current_hash,
        };

        let tx = conn.transaction()?;
        tx.execute(
            "DELETE FROM user_sessions WHERE username = ?1",
            params![current_username],
        )?;
        tx.execute(
            "UPDATE users SET username = ?1, password_hash = ?2 WHERE username = ?3",
            params![next_username, password_hash, current_username],
        )?;
        if current_username != next_username {
            tx.execute(
                "UPDATE annotations SET username = ?1 WHERE username = ?2",
                params![next_username, current_username],
            )?;
            tx.execute(
                "UPDATE video_danmaku SET username = ?1 WHERE username = ?2",
                params![next_username, current_username],
            )?;
        }
        tx.commit()?;
        Ok(Some(next_username.to_string()))
    }

    pub fn delete_public_user(&self, username: &str) -> AppResult<bool> {
        let mut conn = self.conn.lock().expect("db mutex poisoned");
        let tx = conn.transaction()?;
        tx.execute(
            "DELETE FROM user_sessions WHERE username = ?1",
            params![username],
        )?;
        tx.execute(
            "DELETE FROM annotations WHERE username = ?1",
            params![username],
        )?;
        tx.execute(
            "DELETE FROM video_danmaku WHERE username = ?1",
            params![username],
        )?;
        let deleted = tx.execute("DELETE FROM users WHERE username = ?1", params![username])?;
        tx.commit()?;
        Ok(deleted > 0)
    }

    pub fn list_visible_annotations(
        &self,
        note_slug: &str,
        viewer_username: Option<&str>,
    ) -> AppResult<Vec<NoteAnnotation>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        if let Some(username) = viewer_username {
            let mut stmt = conn.prepare(
                r#"
                SELECT id, username, note_slug, start_offset, end_offset, quote, color, comment, visibility, created_at, updated_at
                FROM annotations
                WHERE note_slug = ?1
                  AND (username = ?2 OR (visibility = 'public' AND comment IS NOT NULL))
                ORDER BY start_offset ASC, id ASC
                "#,
            )?;
            let rows = stmt.query_map(params![note_slug, username], annotation_from_row)?;
            Ok(rows.filter_map(Result::ok).collect())
        } else {
            let mut stmt = conn.prepare(
                r#"
                SELECT id, username, note_slug, start_offset, end_offset, quote, color, comment, visibility, created_at, updated_at
                FROM annotations
                WHERE note_slug = ?1
                  AND visibility = 'public'
                  AND comment IS NOT NULL
                ORDER BY start_offset ASC, id ASC
                "#,
            )?;
            let rows = stmt.query_map(params![note_slug], annotation_from_row)?;
            Ok(rows.filter_map(Result::ok).collect())
        }
    }

    pub fn create_annotation(&self, annotation: NewAnnotation<'_>) -> AppResult<NoteAnnotation> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let now = Utc::now().to_rfc3339();
        conn.execute(
            r#"
            INSERT INTO annotations(username, note_slug, start_offset, end_offset, quote, color, comment, visibility, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)
            "#,
            params![
                annotation.username,
                annotation.note_slug,
                annotation.start_offset as i64,
                annotation.end_offset as i64,
                annotation.quote,
                annotation.color,
                annotation.comment,
                annotation.visibility,
                now,
            ],
        )?;
        let id = conn.last_insert_rowid();
        annotation_by_id(&conn, id, annotation.username)?
            .ok_or_else(|| AppError::internal("annotation insert failed"))
    }

    pub fn update_annotation(
        &self,
        id: i64,
        username: &str,
        color: Option<&str>,
        comment: Option<&str>,
        visibility: &str,
    ) -> AppResult<Option<NoteAnnotation>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let updated = conn.execute(
            "UPDATE annotations SET color = ?1, comment = ?2, visibility = ?3, updated_at = ?4 WHERE id = ?5 AND username = ?6",
            params![color, comment, visibility, Utc::now().to_rfc3339(), id, username],
        )?;
        if updated == 0 {
            return Ok(None);
        }
        annotation_by_id(&conn, id, username)
    }

    pub fn update_annotation_by_admin(
        &self,
        id: i64,
        color: Option<&str>,
        comment: Option<&str>,
        visibility: &str,
    ) -> AppResult<Option<NoteAnnotation>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let existing: Option<(String, String)> = conn
            .query_row(
                "SELECT username, note_slug FROM annotations WHERE id = ?1",
                params![id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        let Some((username, _note_slug)) = existing else {
            return Ok(None);
        };
        let updated = conn.execute(
            "UPDATE annotations SET color = ?1, comment = ?2, visibility = ?3, updated_at = ?4 WHERE id = ?5",
            params![color, comment, visibility, Utc::now().to_rfc3339(), id],
        )?;
        if updated == 0 {
            return Ok(None);
        }
        annotation_by_id(&conn, id, &username)
    }

    pub fn delete_annotation(&self, id: i64, username: &str) -> AppResult<bool> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let deleted = conn.execute(
            "DELETE FROM annotations WHERE id = ?1 AND username = ?2",
            params![id, username],
        )?;
        Ok(deleted > 0)
    }

    pub fn delete_annotation_by_admin(&self, id: i64) -> AppResult<bool> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let deleted = conn.execute("DELETE FROM annotations WHERE id = ?1", params![id])?;
        Ok(deleted > 0)
    }

    pub fn list_video_danmaku(
        &self,
        note_slug: &str,
        video_src: &str,
    ) -> AppResult<Vec<VideoDanmaku>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        // 使用 LIKE 进行模糊匹配，以便找回包含旧哈希前缀的记录
        let pattern = format!("%{}%", video_src);
        let mut stmt = conn.prepare(
            r#"
            SELECT id, username, note_slug, video_src, time_ms, body, color, created_at
            FROM video_danmaku
            WHERE note_slug = ?1 AND video_src LIKE ?2
            ORDER BY time_ms ASC, id ASC
            "#,
        )?;
        let rows = stmt.query_map(params![note_slug, pattern], video_danmaku_from_row)?;
        Ok(rows.filter_map(Result::ok).collect())
    }

    pub fn create_video_danmaku(&self, danmaku: NewVideoDanmaku<'_>) -> AppResult<VideoDanmaku> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let now = Utc::now().to_rfc3339();
        conn.execute(
            r#"
            INSERT INTO video_danmaku(username, note_slug, video_src, time_ms, body, color, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![
                danmaku.username,
                danmaku.note_slug,
                danmaku.video_src,
                danmaku.time_ms,
                danmaku.body,
                danmaku.color,
                now,
            ],
        )?;
        let id = conn.last_insert_rowid();
        video_danmaku_by_id(&conn, id)?
            .ok_or_else(|| AppError::internal("video danmaku insert failed"))
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
                created_at: row
                    .get::<_, String>(2)
                    .map(|s| time::format_cst(&s))
                    .unwrap_or_default(),
            })
        })?;
        Ok(rows.filter_map(Result::ok).collect())
    }
}

fn hash_password(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    Ok(Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(AppError::internal)?
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

fn annotation_by_id(
    conn: &Connection,
    id: i64,
    username: &str,
) -> AppResult<Option<NoteAnnotation>> {
    conn.query_row(
        r#"
        SELECT id, username, note_slug, start_offset, end_offset, quote, color, comment, visibility, created_at, updated_at
        FROM annotations
        WHERE id = ?1 AND username = ?2
        "#,
        params![id, username],
        annotation_from_row,
    )
    .optional()
    .map_err(Into::into)
}

fn annotation_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<NoteAnnotation> {
    Ok(NoteAnnotation {
        id: row.get(0)?,
        username: row.get(1)?,
        note_slug: row.get(2)?,
        start_offset: row.get::<_, i64>(3)? as usize,
        end_offset: row.get::<_, i64>(4)? as usize,
        quote: row.get(5)?,
        color: row.get(6)?,
        comment: row.get(7)?,
        visibility: row.get(8)?,
        created_at: row
            .get::<_, String>(9)
            .map(|s| time::format_cst(&s))
            .unwrap_or_default(),
        updated_at: row
            .get::<_, String>(10)
            .map(|s| time::format_cst(&s))
            .unwrap_or_default(),
    })
}

fn video_danmaku_by_id(conn: &Connection, id: i64) -> AppResult<Option<VideoDanmaku>> {
    conn.query_row(
        r#"
        SELECT id, username, note_slug, video_src, time_ms, body, color, created_at
        FROM video_danmaku
        WHERE id = ?1
        "#,
        params![id],
        video_danmaku_from_row,
    )
    .optional()
    .map_err(Into::into)
}

fn video_danmaku_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<VideoDanmaku> {
    Ok(VideoDanmaku {
        id: row.get(0)?,
        username: row.get(1)?,
        note_slug: row.get(2)?,
        video_src: row.get(3)?,
        time_ms: row.get(4)?,
        body: row.get(5)?,
        color: row.get(6)?,
        created_at: row
            .get::<_, String>(7)
            .map(|s| time::format_cst(&s))
            .unwrap_or_default(),
    })
}

fn managed_public_user_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ManagedPublicUser> {
    let username: String = row.get(0)?;
    Ok(ManagedPublicUser {
        route_key: percent_encode_component(&username),
        username,
        created_at: row
            .get::<_, String>(1)
            .map(|s| time::format_cst(&s))
            .unwrap_or_default(),
        session_count: row.get::<_, i64>(2)? as usize,
        annotation_count: row.get::<_, i64>(3)? as usize,
    })
}

fn ensure_annotations_visibility_column(conn: &Connection) -> AppResult<()> {
    let mut stmt = conn.prepare("PRAGMA table_info(annotations)")?;
    let mut rows = stmt.query([])?;
    let mut has_visibility = false;

    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?;
        if name == "visibility" {
            has_visibility = true;
            break;
        }
    }

    if !has_visibility {
        conn.execute(
            "ALTER TABLE annotations ADD COLUMN visibility TEXT NOT NULL DEFAULT 'private'",
            [],
        )?;
    }

    Ok(())
}

fn ensure_video_danmaku_color_column(conn: &Connection) -> AppResult<()> {
    let mut stmt = conn.prepare("PRAGMA table_info(video_danmaku)")?;
    let mut rows = stmt.query([])?;
    let mut has_color = false;

    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?;
        if name == "color" {
            has_color = true;
            break;
        }
    }

    if !has_color {
        conn.execute(
            "ALTER TABLE video_danmaku ADD COLUMN color TEXT NOT NULL DEFAULT '#ffffff'",
            [],
        )?;
    }

    Ok(())
}

fn percent_encode_component(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char)
            }
            _ => {
                encoded.push('%');
                encoded.push(
                    char::from_digit((byte >> 4) as u32, 16)
                        .unwrap()
                        .to_ascii_uppercase(),
                );
                encoded.push(
                    char::from_digit((byte & 0x0F) as u32, 16)
                        .unwrap()
                        .to_ascii_uppercase(),
                );
            }
        }
    }
    encoded
}
