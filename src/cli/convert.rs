use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Args;
use yansi::Paint;

use fujinx::{Camera, Config, Recipe};

use super::parse::parse_path;
use super::progress::Progress;

/// Convert RAF files to JPEG.
#[derive(Args)]
pub struct ConvertCmd {
    /// Input RAF file(s).
    #[arg(required = true, value_parser = parse_path)]
    raw: Vec<PathBuf>,

    /// Recipe name(s).
    #[arg(long, required = true)]
    recipe: Vec<String>,

    /// Output directory.
    #[arg(long, default_value = "out", value_parser = parse_path)]
    out: PathBuf,
}

pub fn convert(args: ConvertCmd) -> Result<()> {
    let config = Config::open()?;

    std::fs::create_dir_all(&args.out)
        .with_context(|| format!("failed to create output directory: {}", args.out.display()))?;

    let recipes: Vec<(String, Recipe)> = args
        .recipe
        .iter()
        .map(|name| {
            let recipe = config.read_recipe(name)?;
            let stem = if name.ends_with(".yaml") {
                Path::new(name)
                    .file_stem()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned()
            } else {
                name.clone()
            };

            Ok((stem, recipe))
        })
        .collect::<Result<_>>()?;

    let mut camera = Camera::open_first()?;

    let progress = Progress::new(args.raw.len() * (recipes.len() + 1));
    for raw_path in &args.raw {
        let raw_stem = raw_path.file_stem().unwrap().to_string_lossy();
        let raw = std::fs::read(raw_path)
            .with_context(|| format!("failed to read RAF: {}", raw_path.display()))?;

        eprintln!(
            "{progress} {camera} Loading {}",
            raw_path.display().paint(fujinx::BLUE),
        );
        let profile = camera.load_raw(&raw)?;

        for (recipe_stem, recipe) in &recipes {
            let out_name = format!("{raw_stem}_{recipe_stem}.jpg");
            let out_path = args.out.join(&out_name);

            eprintln!(
                "{progress} {camera}   Rendering {}",
                out_path.display().paint(fujinx::BLUE),
            );
            let jpg = camera.render(&profile, recipe)?;
            std::fs::write(&out_path, &jpg)
                .with_context(|| format!("failed to write JPEG: {}", out_path.display()))?;
        }
    }
    eprintln!(
        "{} Rendered {} images in {}",
        crate::done(),
        (args.raw.len() * recipes.len()).paint(fujinx::BLUE),
        format!("{:.1}s", progress.elapsed()).paint(fujinx::BLUE),
    );

    Ok(())
}
