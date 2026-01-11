use anyhow::{Ok, Result};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

mod apply;
mod classify;
mod deduplicate;
mod dvd;
mod manifest;
mod photo;
mod plan;
mod report;
mod time;
mod video;

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let cmd = args.next().unwrap_or_else(|| "help".to_string());

    match cmd.as_str() {
        "plan" => {
            let root = PathBuf::from(args.next().unwrap_or_else(|| ".".to_string()));
            let out_root = PathBuf::from(args.next().unwrap_or_else(|| "./ExportSet".to_string()));

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
            println!("Duplicate photos:    {}", summary.duplicate_photos);
            println!("Duplicate videos:    {}", summary.duplicate_videos);
            println!("Out root:            {}", out_root.display());
            println!("Wrote:               manifest.jsonl");
        }
        "apply" => {
            let manifest =
                PathBuf::from(args.next().unwrap_or_else(|| "manifest.jsonl".to_string()));
            let items = manifest::read_manifest_jsonl(&manifest)?;
            let summary = apply::apply_items(&items)?;

            println!("Applied manifest:     {}", manifest.display());
            println!("Total:                {}", summary.total);
            println!("Copied:               {}", summary.copied);
            println!("Converted videos:     {}", summary.converted_video);
            println!("Converted DVDs:       {}", summary.converted_dvd);
            println!("Skipped existing:     {}", summary.skipped_existing);
            println!("Skipped duplicate:    {}", summary.skipped_dupliace);
            println!("Failed:               {}", summary.failed);
            println!("Logs: apply_ok.log, apply_fail.log, apply_duplicates_skipped.log");
        }
        "report" => {
            let manifest =
                PathBuf::from(args.next().unwrap_or_else(|| "manifest.jsonl".to_string()));
            let validate_outputs = args.next().as_deref() == Some("--validate-outputs");

            let items = manifest::read_manifest_jsonl(&manifest)?;
            let (summary, notes) = report::build_report(&items, validate_outputs)?;
            report::print_report(&summary, &notes, validate_outputs);

            println!("\nManifest: {}", manifest.display());
            println!(
                "Tip: run `cargo run -- report manifest.jsonl --validate-outputs` after apply."
            );
        }
        _ => {
            eprintln!("Usage:");
            eprintln!(" cargo run -- plan <input_root> <out_root>");
            eprintln!(" cargo run -- apply [manifest.jsonl]");
        }
    }

    Ok(())
}
