use chrono::{DateTime, FixedOffset, Utc};

/// UTC+8 固定偏移（东八区）
fn cst() -> FixedOffset {
    FixedOffset::east_opt(8 * 3600).expect("固定偏移量合法")
}

/// 返回当前东八区时间的 RFC3339 字符串（用于写入数据库）
pub fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

/// 将 RFC3339 字符串转换为东八区的可读格式（用于页面展示）
/// 格式：2024-01-15 14:30
/// 若解析失败则原样返回
pub fn format_cst(rfc3339: &str) -> String {
    match DateTime::parse_from_rfc3339(rfc3339) {
        Ok(dt) => dt.with_timezone(&cst()).format("%Y-%m-%d %H:%M").to_string(),
        Err(_) => rfc3339.to_owned(),
    }
}

/// 返回当前东八区时间，已格式化为可读字符串
pub fn now_cst_display() -> String {
    Utc::now()
        .with_timezone(&cst())
        .format("%Y-%m-%d %H:%M")
        .to_string()
}
