use chrono::NaiveDate;
use serde::Serialize;
use std::collections::HashMap;
use crate::scanner::ImageEntry;
use crate::exif;

#[derive(Debug, Serialize)]
pub struct InferResult {
    pub date: Option<String>,
    pub count: usize,
    pub total: usize,
    pub has_conflict: bool,
    pub conflict_message: Option<String>,
    pub date_entries: Vec<DateEntry>,
}

#[derive(Debug, Serialize)]
pub struct DateEntry {
    pub filename: String,
    pub date: Option<String>,
    pub is_outlier: bool,
}

/// 推断代表日期：众数优先，并列选最早
pub fn infer_date(_folder_path: &str, entries: &[ImageEntry]) -> InferResult {
    let total = entries.len();
    let mut date_entries = Vec::new();
    let mut date_counts: HashMap<String, i32> = HashMap::new();

    for entry in entries {
        let date = exif::extract_date(&entry.path);
        let date_str = date.map(|d| d.format("%Y-%m-%d").to_string());

        if let Some(ref ds) = date_str {
            *date_counts.entry(ds.clone()).or_insert(0) += 1;
        }

        date_entries.push(DateEntry {
            filename: entry.filename.clone(),
            date: date_str.clone(),
            is_outlier: false,
        });
    }

    // 找出众数日期
    let (representative, _max_count) = date_counts.iter()
        .max_by(|a, b| {
            a.1.cmp(b.1).then_with(|| {
                let da = NaiveDate::parse_from_str(a.0, "%Y-%m-%d").unwrap_or(NaiveDate::MAX);
                let db = NaiveDate::parse_from_str(b.0, "%Y-%m-%d").unwrap_or(NaiveDate::MAX);
                da.cmp(&db)
            })
        })
        .map(|(d, c)| (Some(d.clone()), *c as usize))
        .unwrap_or((None, 0));

    // 检查离群日期（与众数日期相差 > 30 天）
    let mut has_conflict = false;
    let mut conflict_msg = None;

    if let Some(ref rep_date) = representative {
        if let Ok(rep_parsed) = NaiveDate::parse_from_str(rep_date, "%Y-%m-%d") {
            let mut outlier_count = 0;
            for de in &mut date_entries {
                if let Some(ref d) = de.date {
                    if d != rep_date {
                        if let Ok(parsed) = NaiveDate::parse_from_str(d, "%Y-%m-%d") {
                            let days_diff = (parsed - rep_parsed).num_days().abs();
                            if days_diff > 30 {
                                de.is_outlier = true;
                                outlier_count += 1;
                                has_conflict = true;
                            }
                        }
                    }
                }
            }
            if outlier_count > 0 {
                conflict_msg = Some(format!(
                    "有 {} 张图片日期与众数相差超过 30 天，可能不是同一事件",
                    outlier_count
                ));
            }
        }
    }

    let count = entries.iter().filter(|e| {
        date_entries.iter().any(|de| de.filename == e.filename && de.date.is_some())
    }).count();

    InferResult {
        date: representative,
        count,
        total,
        has_conflict,
        conflict_message: conflict_msg,
        date_entries,
    }
}