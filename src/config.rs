use std::{env, fs, path::PathBuf};

use crate::error::AppResult;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub base_url: String,
    pub site_name: String,
    pub content_dir: PathBuf,
    pub notes_dir: PathBuf,
    pub assets_dir: PathBuf,
    pub generated_dir: PathBuf,
    pub generated_assets_dir: PathBuf,
    pub data_dir: PathBuf,
    pub admin_username: String,
    pub admin_password: String,
    pub watch_enabled: bool,
    pub upload_limit_mb: usize,
}

impl AppConfig {
    pub fn from_env() -> AppResult<Self> {
        let content_dir =
            PathBuf::from(env::var("M2W_CONTENT_DIR").unwrap_or_else(|_| "content".into()));
        let generated_dir = PathBuf::from(
            env::var("M2W_GENERATED_DIR").unwrap_or_else(|_| "generated/site".into()),
        );
        let data_dir = PathBuf::from(env::var("M2W_DATA_DIR").unwrap_or_else(|_| "data".into()));
        let notes_dir = content_dir.join("notes");
        let assets_dir = content_dir.join("assets");
        let generated_assets_dir = generated_dir.join("assets");

        Ok(Self {
            host: env::var("M2W_HOST").unwrap_or_else(|_| "127.0.0.1".into()),
            port: env::var("M2W_PORT")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(3000),
            base_url: env::var("M2W_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:3000".into()),
            site_name: env::var("M2W_SITE_NAME").unwrap_or_else(|_| "markdown2web".into()),
            content_dir,
            notes_dir,
            assets_dir,
            generated_dir,
            generated_assets_dir,
            data_dir,
            admin_username: env::var("M2W_ADMIN_USERNAME").unwrap_or_else(|_| "admin".into()),
            admin_password: env::var("M2W_ADMIN_PASSWORD")
                .unwrap_or_else(|_| "Pcsensor1121@".into()),
            watch_enabled: env::var("M2W_WATCH_ENABLED")
                .map(|v| !matches!(v.as_str(), "0" | "false" | "off"))
                .unwrap_or(true),
            upload_limit_mb: env::var("M2W_UPLOAD_LIMIT_MB")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(10),
        })
    }

    pub fn ensure_directories(&self) -> AppResult<()> {
        fs::create_dir_all(&self.notes_dir)?;
        fs::create_dir_all(&self.assets_dir)?;
        fs::create_dir_all(&self.generated_assets_dir)?;
        fs::create_dir_all(&self.data_dir)?;
        Ok(())
    }

    pub fn database_path(&self) -> PathBuf {
        self.data_dir.join("app.db")
    }
}
