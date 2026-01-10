use anyhow::Result;
use std::{
    collections::HashMap,
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
};

fn blake3_hash_file(path: &Path) -> io::Result<blake3::Hash> {
    let mut f = File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buf = [0u8; 1024 * 1024];

    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    Ok(hasher.finalize())
}

pub fn find_exact_duplicates(paths: &[PathBuf]) -> Result<HashMap<String, Vec<PathBuf>>> {
    let mut by_size: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    for p in paths {
        let size = std::fs::metadata(p).map(|m| m.len()).unwrap_or(0);
        by_size.entry(size).or_default().push(p.clone());
    }

    let mut dup_groups: HashMap<String, Vec<PathBuf>> = HashMap::new();

    for (_size, group) in by_size {
        if group.len() < 2 {
            continue;
        }

        let mut by_hash: HashMap<String, Vec<PathBuf>> = HashMap::new();
        for p in group {
            let h = blake3_hash_file(&p)?.to_hex().to_string();
            by_hash.entry(h).or_default().push(p);
        }

        for (h, g) in by_hash {
            if g.len() > 1 {
                dup_groups.entry(h).insert_entry(g);
            }
        }
    }

    Ok(dup_groups)
}
