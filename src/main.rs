use std::collections::HashMap;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Clone, Copy)]
enum Kind {
    Photo,
    Video,
    Dvd,
    Ignore,
}

fn normalize_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
}

fn classify(path: &Path) -> Kind {
    let extension = normalize_extension(path);
    match extension.as_deref() {
        Some("jpg") | Some("jpeg") | Some("png") => Kind::Photo,
        Some("mp4") | Some("avi") | Some("mov") | Some("m4v") => Kind::Video,
        Some("vob") | Some("ifo") | Some("bup") => Kind::Dvd,
        _ => Kind::Ignore,
    }
}

fn print_top(title: &str, map: &HashMap<String, u64>, n: usize) {
    let mut v: Vec<(&String, &u64)> = map.iter().collect();
    v.sort_by(|a, b| b.1.cmp(a.1));

    println!("{title}:");
    for (i, (extension, count)) in v.into_iter().take(n).enumerate() {
        println!("{:>2}. {:<10} {}", i + 1, extension, count);
    }
}

fn main() {
    let root = std::env::args().nth(1).unwrap_or_else(|| ".".to_string());

    let mut photos = 0u64;
    let mut videos = 0u64;
    let mut dvds = 0u64;
    let mut ignored = 0u64;

    for entry in WalkDir::new(&root) {
        let entry = match entry {
            Ok(e) => e,
            Err(err) => {
                eprintln!("Walk error: {err}");
                continue;
            }
        };

        // check if it is a file
        if !entry.file_type().is_file() {
            continue;
        }

        match classify(entry.path()) {
            Kind::Photo => photos += 1,
            Kind::Video => videos += 1,
            Kind::Dvd => dvds += 1,
            Kind::Ignore => ignored += 1,
        }
    }

    println!("Scanned: {root}");
    println!("Photos: {photos}");
    println!("Videos: {videos}");
    println!("DVD files: {dvds}");
    println!("Ignored: {ignored}");
}
