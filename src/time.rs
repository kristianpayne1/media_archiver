use anyhow::Result;
use chrono::{DateTime, Local, NaiveDateTime};
use serde::Serialize;
use std::{path::Path, time::SystemTime};

use crate::{
    classify::{Kind, classify, is_jpeg},
    photo_exif, video_meta,
};

#[derive(Debug, Clone, Copy, Serialize)]
pub enum DateSource {
    Exif,
    Ffprobe,
    Mtime,
    None,
}

pub fn file_mtime(path: &Path) -> Option<NaiveDateTime> {
    let meta = std::fs::metadata(path).ok()?;
    let modified: SystemTime = meta.modified().ok()?;
    let dt: DateTime<Local> = modified.into();
    Some(dt.naive_local())
}

pub fn format_dt(dt: NaiveDateTime) -> String {
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn best_datetime_for_file(path: &Path) -> Result<(Option<NaiveDateTime>, DateSource)> {
    match classify(path) {
        Kind::Photo => {
            if is_jpeg(path) {
                if let Some(dt) = photo_exif::exif_capture_datetime(path)? {
                    return Ok((Some(dt), DateSource::Exif));
                }
            }
            if let Some(dt) = file_mtime(path) {
                return Ok((Some(dt), DateSource::Mtime));
            }
            Ok((None, DateSource::None))
        }
        Kind::Video => {
            if let Some(dt) = video_meta::ffprobe_creation_time(path)? {
                return Ok((Some(dt), DateSource::Ffprobe));
            }
            if let Some(dt) = file_mtime(path) {
                return Ok((Some(dt), DateSource::Mtime));
            }
            Ok((None, DateSource::None))
        }
        _ => Ok((None, DateSource::None)),
    }
}

pub fn best_datetime_for_dvd(dvd_root: &Path) -> (Option<NaiveDateTime>, DateSource) {
    if let Some(dt) = file_mtime(dvd_root) {
        return (Some(dt), DateSource::Mtime);
    }
    (None, DateSource::None)
}
