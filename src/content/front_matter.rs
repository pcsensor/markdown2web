use serde::{Deserialize, Serialize};

use crate::{error::AppError, error::AppResult};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FrontMatter {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub summary: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub status: Option<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
}

pub fn parse_front_matter(raw: &str) -> AppResult<(FrontMatter, String)> {
    if !raw.starts_with("---\n") {
        return Ok((FrontMatter::default(), raw.to_string()));
    }

    let mut parts = raw.splitn(3, "---\n");
    let _ = parts.next();
    let yaml = parts.next().unwrap_or_default();
    let rest = parts.next().unwrap_or_default();
    let front_matter = serde_yaml::from_str::<FrontMatter>(yaml)?;
    Ok((front_matter, rest.to_string()))
}

pub fn compose_markdown(front_matter: &FrontMatter, body: &str) -> AppResult<String> {
    let yaml = serde_yaml::to_string(front_matter).map_err(AppError::internal)?;
    Ok(format!("---\n{}---\n{}", yaml, body.trim_start()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_front_matter_block() {
        let (front_matter, body) = parse_front_matter(
            r#"---
title: Hello
slug: hello
tags: [rust, notes]
status: published
---
Body
"#,
        )
        .unwrap();

        assert_eq!(front_matter.title.as_deref(), Some("Hello"));
        assert_eq!(front_matter.slug.as_deref(), Some("hello"));
        assert_eq!(front_matter.tags, vec!["rust", "notes"]);
        assert_eq!(body.trim(), "Body");
    }
}
