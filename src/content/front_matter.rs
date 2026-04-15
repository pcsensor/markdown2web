use serde::{Deserialize, Deserializer, Serialize};

use crate::{error::AppError, error::AppResult};

fn deserialize_string_or_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrVec {
        One(String),
        Many(Vec<String>),
    }

    let opt = Option::<StringOrVec>::deserialize(deserializer).map_err(serde::de::Error::custom)?;
    match opt {
        Some(StringOrVec::One(s)) => Ok(vec![s]),
        Some(StringOrVec::Many(v)) => Ok(v),
        None => Ok(vec![]),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FrontMatter {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub summary: Option<String>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_string_or_vec")]
    pub category: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub status: Option<String>,
    pub updated: Option<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
}

pub fn parse_front_matter(raw: &str) -> AppResult<(FrontMatter, String)> {
    // 统一处理换行符，将 \r\n 替换为 \n 进行分割判断
    // 但为了保持正文原始性，我们只在查找分隔符时使用灵活匹配
    
    if !raw.starts_with("---") {
        return Ok((FrontMatter::default(), raw.to_string()));
    }

    // 查找第一个换行符，确定第一行是否只是 "---" (允许后面有空格)
    let first_line_end = raw.find('\n').unwrap_or(raw.len());
    let first_line = raw[..first_line_end].trim_end();
    if first_line != "---" {
        return Ok((FrontMatter::default(), raw.to_string()));
    }

    // 查找第二个 "---" 分隔符
    // 它必须在一行的开头
    let search_start = first_line_end + 1;
    if search_start >= raw.len() {
        return Ok((FrontMatter::default(), raw.to_string()));
    }

    // 我们寻找 "\n---" 序列
    let mut found_end = None;
    let bytes = raw.as_bytes();
    for i in search_start..bytes.len() {
        if bytes[i] == b'\n' && i + 1 < bytes.len() && raw[i+1..].starts_with("---") {
            let line_after = &raw[i+1..];
            let next_line_end = line_after.find('\n').unwrap_or(line_after.len());
            if line_after[..next_line_end].trim_end() == "---" {
                found_end = Some((i, i + 1 + next_line_end));
                break;
            }
        }
    }

    if let Some((yaml_end_in_raw, rest_start)) = found_end {
        let yaml = &raw[search_start..yaml_end_in_raw];
        let rest = &raw[rest_start..];
        
        let front_matter = serde_yaml::from_str::<FrontMatter>(yaml)
            .unwrap_or_else(|_| FrontMatter::default());
        
        // 如果解析出来的 slug 有 \r，去掉它（防御性编程）
        let mut fm = front_matter;
        if let Some(ref mut slug) = fm.slug {
            *slug = slug.trim().to_string();
        }
        
        Ok((fm, rest.trim_start_matches(|c| c == '\r' || c == '\n').to_string()))
    } else {
        Ok((FrontMatter::default(), raw.to_string()))
    }
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
