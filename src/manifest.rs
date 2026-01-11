use crate::plan::PlannedItem;
use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub fn read_manifest_jsonl(path: &Path) -> Result<Vec<PlannedItem>> {
    let file = File::open(path).with_context(|| format!("open manifest {}", path.display()))?;
    let reader = BufReader::new(file);

    let mut items = Vec::new();
    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let item: PlannedItem =
            serde_json::from_str(&line).with_context(|| format!("parse json on line {}", i + 1))?;
        items.push(item);
    }

    Ok(items)
}
