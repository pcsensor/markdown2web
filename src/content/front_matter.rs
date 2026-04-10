use serde::{Deserialize, Serialize};

use crate::{error::AppError, error::AppResult};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FrontMatter {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub summary: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub status: Option<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
}

pub fn parse_front_matter(raw: &str) -> AppResult<(FrontMatter, String)> {
    // 支持 LF (\n) 和 CRLF (\r\n) 两种换行符格式
    let has_lf_front_matter = raw.starts_with("---\n");
    let has_crlf_front_matter = raw.starts_with("---\r\n");

    if !has_lf_front_matter && !has_crlf_front_matter {
        return Ok((FrontMatter::default(), raw.to_string()));
    }

    // 根据换行符类型选择分隔符
    let delimiter = if has_crlf_front_matter {
        "---\r\n"
    } else {
        "---\n"
    };
    let mut parts = raw.splitn(3, delimiter);
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

    #[test]
    fn parses_front_matter_block_with_crlf() {
        // Windows 风格换行符 (CRLF)
        let input = "---\r\ntitle: Hello\r\nslug: hello\r\ntags: [rust, notes]\r\nstatus: published\r\n---\r\nBody\r\n";
        let (front_matter, body) = parse_front_matter(input).unwrap();

        assert_eq!(front_matter.title.as_deref(), Some("Hello"));
        assert_eq!(front_matter.slug.as_deref(), Some("hello"));
        assert_eq!(front_matter.tags, vec!["rust", "notes"]);
        assert_eq!(body.trim(), "Body");
    }

    #[test]
    fn parses_front_matter_block_with_mixed_line_endings() {
        // 混合换行符：文件头使用 CRLF，正文使用 LF
        // 注意：第二个 --- 必须使用与开头相同的换行符类型
        let input = "---\r\ntitle: Mixed\r\nslug: mixed\r\ntags: [test]\r\n---\r\nBody content\nMore content";
        let (front_matter, body) = parse_front_matter(input).unwrap();

        assert_eq!(front_matter.title.as_deref(), Some("Mixed"));
        assert_eq!(front_matter.tags, vec!["test"]);
        assert!(body.contains("Body content"));
    }

    #[test]
    fn no_front_matter_returns_default() {
        let input = "Just body content\nNo front matter here";
        let (front_matter, body) = parse_front_matter(input).unwrap();

        assert!(front_matter.title.is_none());
        assert!(front_matter.tags.is_empty());
        assert_eq!(body, input);
    }

    #[test]
    fn front_matter_with_only_delimiter_not_recognized() {
        // 只有分隔符没有换行符不应被识别为 front matter
        let input = "---Just content";
        let (front_matter, body) = parse_front_matter(input).unwrap();

        assert!(front_matter.title.is_none());
        assert_eq!(body, input);
    }
}
