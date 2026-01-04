use anyhow::Result;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

mod classify;
mod dvd;
mod photo_exif;
mod time;
mod video_meta;

use classify::{Kind, classify, is_jpeg};

fn main() -> Result<()> {
    let root = std::env::args().nth(1).unwrap_or_else(|| ".".to_string());
    let root_path = PathBuf::from(root);

    let mut dvd_roots: HashSet<PathBuf> = HashSet::new();

    let mut photos = 0u64;
    let mut photos_with_date = 0u64;
    let mut videos = 0u64;
    let mut ignored = 0u64;

    for entry in WalkDir::new(&root_path) {
        let entry = match entry {
            Ok(e) => e,
            Err(err) => {
                eprintln!("Walk error: {err}");
                continue;
            }
        };

        let path = entry.path();

        if entry.file_type().is_dir() {
            if let Some(dvd_root) = dvd::dvd_root_from_video_ts_dir(path) {
                dvd_roots.insert(dvd_root);
            }
            continue;
        }

        if dvd::is_inside_video_ts(path) {
            ignored += 1;
            continue;
        }

        if !entry.file_type().is_file() {
            continue;
        }

        match classify(path) {
            Kind::Photo => {
                photos += 1;

                if is_jpeg(path) {
                    match photo_exif::exif_capture_datetime(path) {
                        Ok(Some(dt)) => {
                            photos_with_date += 1;
                            println!(
                                "(photo) {}    {}",
                                dt.format("%Y-%m-%d %H:%M:%S"),
                                path.display()
                            );
                        }
                        Ok(None) => println!("(photo) (no exif date)    {}", path.display()),
                        Err(err) => println!("(photo) (exif error) {} [ {err} ]", path.display()),
                    }
                }
            }
            Kind::Video => {
                videos += 1;
                match video_meta::video_best_datetime(path) {
                    Ok(Some(dt)) => println!(
                        "(video) {}    {}",
                        dt.format("%Y-%m-%d %H:%M:%S"),
                        path.display()
                    ),
                    Ok(None) => println!("(video) (no date)    {}", path.display()),
                    Err(err) => println!("(video) (error) {} [ {err} ]", path.display()),
                }
            }
            Kind::Ignore => ignored += 1,
        }
    }

    let mut dvd_count = 0u64;
    for dvd_root in dvd_roots.iter() {
        dvd_count += 1;
        let dt = dvd::dvd_best_datetime(dvd_root);
        let vobs = dvd::dvd_main_title_vobs(dvd_root)?;

        match dt {
            Some(dt) => println!(
                "(dvd)  {}  {}  ({} VOBs)",
                dt.format("%Y-%m-%d %H:%M:%S"),
                dvd_root.display(),
                vobs.len()
            ),
            None => println!(
                "(dvd)  (no date)  {}  ({} VOBs)",
                dvd_root.display(),
                vobs.len()
            ),
        }
    }

    println!();
    println!("Scanned: {}", root_path.display());
    println!("Photos: {photos}");
    println!("With EXIF date: {photos_with_date}");
    println!("Videos: {videos}");
    println!("DVDs (as items): {dvd_count}");
    println!("Ignored: {ignored}");
    Ok(())
}
