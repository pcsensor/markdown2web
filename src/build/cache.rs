use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BuildCache {
    #[serde(default)]
    pub note_hashes: HashMap<String, String>,
    #[serde(default)]
    pub media_job_destinations: HashSet<String>,
}

impl BuildCache {
    pub fn load(path: &Path) -> AppResult<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = fs::read_to_string(path)?;
        serde_json::from_str(&raw).map_err(AppError::internal)
    }

    pub fn save(&self, path: &Path) -> AppResult<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let raw = serde_json::to_string_pretty(self).map_err(AppError::internal)?;
        fs::write(path, raw)?;
        Ok(())
    }

    pub fn changed_count<'a>(&self, incoming: impl Iterator<Item = (&'a str, &'a str)>) -> usize {
        incoming
            .filter(|(slug, hash)| self.note_hashes.get(*slug).map(String::as_str) != Some(*hash))
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_changed_hashes() {
        let mut cache = BuildCache::default();
        cache.note_hashes.insert("a".into(), "1".into());
        let changed = cache.changed_count([("a", "2"), ("b", "1")].into_iter());
        assert_eq!(changed, 2);
    }
}
