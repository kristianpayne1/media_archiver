use anyhow::Result;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

mod classify;
mod dvd;
mod photo_exif;
mod plan;
mod time;
mod video_meta;

fn main() -> Result<()> {
    let root = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| ".".to_string()));
    let out_root = PathBuf::from(
        std::env::args()
            .nth(2)
            .unwrap_or_else(|| "./ExportSet".to_string()),
    );

    let (items, summary) = plan::build_plan(&root, &out_root)?;

    let mut f = File::create("manifest.jsonl")?;
    for item in items {
        writeln!(f, "{}", serde_json::to_string(&item)?)?;
    }

    println!("Planned items:       {}", summary.planned);
    println!("Photos:              {}", summary.photos);
    println!("Videos:              {}", summary.videos);
    println!("DVDs:                {}", summary.dvds);
    println!("Missing date:        {}", summary.missing_date);
    println!("Need convert (video):{}", summary.need_convert_video);
    println!("Need convert (dvd):  {}", summary.need_convert_dvd);
    println!("Out root:            {}", out_root.display());
    println!("Wrote:               manifest.jsonl");

    Ok(())
}
