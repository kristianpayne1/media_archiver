use crate::plan::{Action, MediaKind, PlannedItem};
use anyhow::{Ok, Result};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug)]
pub struct ReportSummary {
    pub total: u64,
    pub by_kind: BTreeMap<String, u64>,
    pub by_action: BTreeMap<String, u64>,
    pub by_date_source: BTreeMap<String, u64>,
    pub missing_date: u64,
    pub duplicates: u64,
    pub by_year: BTreeMap<String, u64>,
    pub by_year_month: BTreeMap<String, u64>,

    pub outputs_exist: u64,
    pub outputs_missing: u64,
    pub outputs_zero_bytes: u64,
}

impl ReportSummary {
    pub fn new() -> Self {
        Self {
            total: 0,
            by_kind: BTreeMap::new(),
            by_action: BTreeMap::new(),
            by_date_source: BTreeMap::new(),
            missing_date: 0,
            duplicates: 0,
            by_year: BTreeMap::new(),
            by_year_month: BTreeMap::new(),
            outputs_exist: 0,
            outputs_missing: 0,
            outputs_zero_bytes: 0,
        }
    }
}

fn bump(map: &mut BTreeMap<String, u64>, key: impl Into<String>) {
    *map.entry(key.into()).or_insert(0) += 1;
}

fn kind_str(k: MediaKind) -> &'static str {
    match k {
        MediaKind::Photo => "Photo",
        MediaKind::Video => "Video",
        MediaKind::Dvd => "Dvd",
    }
}

fn action_str(a: Action) -> &'static str {
    match a {
        Action::Copy => "Copy",
        Action::ConvertVideo => "ConvertVideo",
        Action::ConvertDvd => "ConvertDvd",
    }
}

fn year_from_best_dt(best_dt: &Option<String>) -> Option<String> {
    best_dt
        .as_ref()
        .and_then(|s| s.get(0..4))
        .map(|s| s.to_string())
}

fn year_month_from_best_dt(best_dt: &Option<String>) -> Option<String> {
    best_dt
        .as_ref()
        .and_then(|s| s.get(0..7))
        .map(|s| s.to_string())
}

pub fn build_report(
    items: &[PlannedItem],
    validate_outputs: bool,
) -> Result<(ReportSummary, Vec<String>)> {
    let mut s = ReportSummary::new();
    let mut notes: Vec<String> = Vec::new();

    let mut missing_dates: Vec<&PlannedItem> = Vec::new();
    let mut duplicates: Vec<&PlannedItem> = Vec::new();
    let mut missing_outputs: Vec<&PlannedItem> = Vec::new();

    for item in items {
        s.total += 1;
        bump(&mut s.by_kind, kind_str(item.kind));
        bump(&mut s.by_action, action_str(item.action));
        bump(&mut s.by_date_source, format!("{:?}", item.date_source));

        if item.best_dt.is_none() {
            s.missing_date += 1;
            missing_dates.push(item);
        }

        if item.duplicate_of.is_some() {
            s.duplicates += 1;
            duplicates.push(item);
        }

        if let Some(y) = year_from_best_dt(&item.best_dt) {
            bump(&mut s.by_year, y);
        } else {
            bump(&mut s.by_year, "UnknownDate");
        }

        if let Some(ym) = year_month_from_best_dt(&item.best_dt) {
            bump(&mut s.by_year_month, ym);
        } else {
            bump(&mut s.by_year_month, "UnknownDate");
        }

        if validate_outputs {
            let dst = PathBuf::from(&item.dst);
            if dst.exists() {
                s.outputs_exist += 1;
                let size = std::fs::metadata(&dst).map(|m| m.len()).unwrap_or(0);
                if size == 0 {
                    s.outputs_zero_bytes += 1;
                }
            } else if item.duplicate_of.is_none() {
                s.outputs_missing += 1;
                missing_outputs.push(item);
            }
        }
    }

    if !missing_dates.is_empty() {
        notes.push("Missing dates:".to_string());
        for it in missing_dates {
            notes.push(format!(
                "    - {:?} {:?} src={} (date_source={:?}",
                it.kind, it.action, it.src, it.date_source
            ));
        }
    }

    if !duplicates.is_empty() {
        notes.push("Duplicate files (skipped in apply):".to_string());
        for it in duplicates {
            notes.push(format!(
                "    - {:?} src={} dup_of {}",
                it.kind,
                it.src,
                it.duplicate_of.as_deref().unwrap_or("?")
            ));
        }
    }

    if validate_outputs && !missing_outputs.is_empty() {
        notes.push("Missing output (dst does not exist):".to_string());
        for it in missing_outputs {
            notes.push(format!(
                "    - {:?} {:?} dst={} (src={})",
                it.kind, it.action, it.dst, it.src
            ));
        }
    }

    Ok((s, notes))
}

pub fn print_report(summary: &ReportSummary, notes: &[String], validate_outputs: bool) {
    println!("=== Manifest Report ===");
    println!("Total planned items: {}", summary.total);

    println!("\nBy kind:");
    for (k, v) in &summary.by_kind {
        println!("  {k:12} {v}");
    }

    println!("\nBy action:");
    for (k, v) in &summary.by_action {
        println!("  {k:12} {v}");
    }

    println!("\nBy date source:");
    for (k, v) in &summary.by_date_source {
        println!("  {k:12} {v}");
    }

    println!("\nMissing date: {}", summary.missing_date);
    println!("Duplicates (input): {}", summary.duplicates);

    // Show “top-ish” years/months (BTreeMap is sorted; that's fine for browsing)
    println!("\nBy year (sorted):");
    for (k, v) in &summary.by_year {
        println!("  {k:12} {v}");
    }

    println!("\nBy year-month (sorted):");
    for (k, v) in &summary.by_year_month {
        println!("  {k:12} {v}");
    }

    if validate_outputs {
        println!("\nOutput validation:");
        println!("  Outputs exist:      {}", summary.outputs_exist);
        println!("  Outputs missing:    {}", summary.outputs_missing);
        println!("  Outputs zero-bytes: {}", summary.outputs_zero_bytes);
    }

    if !notes.is_empty() {
        println!("\n=== Notes ===");
        for line in notes {
            println!("{line}");
        }
    }
}
