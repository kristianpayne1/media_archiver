use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub fn is_inside_video_ts(path: &Path) -> bool {
    path.components().any(|c| {
        c.as_os_str()
            .to_str()
            .map(|s| s.eq_ignore_ascii_case("VIDEO_TS"))
            .unwrap_or(false)
    })
}

pub fn dvd_root_from_video_ts_dir(path: &Path) -> Option<PathBuf> {
    let name = path.file_name()?.to_str()?;
    if name.eq_ignore_ascii_case("VIDEO_TS") {
        return path.parent().map(|p| p.to_path_buf());
    }
    None
}

pub fn dvd_main_title_vobs(dvd_root: &Path) -> Result<Vec<PathBuf>> {
    let video_ts = dvd_root.join("VIDEO_TS");
    if !video_ts.is_dir() {
        return Ok(vec![]);
    }

    let mut groups: HashMap<String, Vec<PathBuf>> = HashMap::new();

    for entry in std::fs::read_dir(&video_ts)? {
        let entry = entry?;
        let path = entry.path();

        let is_vob = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.eq_ignore_ascii_case("vob"))
            .unwrap_or(false);

        if !is_vob {
            continue;
        }

        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if !name.to_ascii_uppercase().starts_with("VTS_")
            || name.len() < 10
            || name.get(7..9) == Some("_0")
        {
            continue;
        }

        let key = name.get(0..6).unwrap().to_string();
        groups.entry(key).or_default().push(path);
    }

    let mut best: Vec<PathBuf> = vec![];
    let mut best_size = 0u64;

    for (_, mut files) in groups {
        files.sort();

        let size: u64 = files
            .iter()
            .map(|p| p.metadata().map(|m| m.len()).unwrap_or(0))
            .sum();

        if size > best_size {
            best_size = size;
            best = files;
        }
    }

    Ok(best)
}
