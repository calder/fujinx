use anyhow::Result;
use clap::{Args, Subcommand};
use yansi::Paint;

use fujinx::{Camera, Config, RecipeSource};

/// Manage recipes.
#[derive(Args)]
pub struct RecipeCmd {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Show a recipe from a custom slot on the camera.
    Show(ShowArgs),

    /// Save a recipe from a custom slot to ~/.fujinx/recipes/<name>.yaml.
    Save(SaveArgs),

    /// Load a recipe from ~/.fujinx/recipes/<name>.yaml into a custom slot.
    Load(LoadArgs),

    /// Delete a recipe from ~/.fujinx/recipes/.
    Del(DelArgs),

    /// List available recipes.
    List,
}

#[derive(Args)]
struct ShowArgs {
    /// Custom slot.
    #[arg(short)]
    c: Option<u8>,

    /// Recipe name.
    name: Option<String>,
}

#[derive(Args)]
struct SaveArgs {
    /// Custom slot.
    #[arg(short)]
    c: u8,

    /// Recipe name.
    name: String,

    /// Overwrite an existing recipe.
    #[arg(long)]
    force: bool,
}

#[derive(Args)]
struct LoadArgs {
    /// Custom slot.
    #[arg(short)]
    c: u8,

    /// Recipe name.
    name: String,
}

#[derive(Args)]
struct DelArgs {
    /// Recipe name.
    name: String,
}

pub fn recipe(args: RecipeCmd) -> Result<()> {
    match args.command {
        Command::Show(args) => show(args),
        Command::Save(args) => save(args),
        Command::Load(args) => load(args),
        Command::Del(args) => del(args),
        Command::List => list(),
    }
}

fn label(s: &str) -> String {
    format!("{:>14}", s)
}

fn fmt_i32(n: i32) -> String {
    if n == 0 {
        "0".to_string()
    } else {
        format!("{n:+}")
    }
}

fn fmt_f64(n: f64) -> String {
    if n == 0.0 {
        "0".to_string()
    } else if n.fract() == 0.0 {
        format!("{:+}", n as i32)
    } else {
        format!("{n:+}")
    }
}

fn print_recipe(r: &fujinx::Recipe) {
    println!(
        "  {} {}",
        label("Name:").dim(),
        r.name.paint(fujinx::BLUE).bold()
    );
    println!("  {} {}", label("Film:").dim(), r.film);
    println!("  {} {}", label("Grain:").dim(), r.grain);
    println!(
        "  {} RG: {}, B: {}",
        label("Color Chrome:").dim(),
        r.color_chrome,
        r.color_chrome_blue
    );

    let wb_shift = match (r.white_balance_red, r.white_balance_blue) {
        (0, 0) => String::new(),
        (red, blue) => format!(" (R{red:+}, B{blue:+})"),
    };
    println!(
        "  {} {}{}",
        label("White Balance:").dim(),
        r.white_balance,
        wb_shift
    );
    println!(
        "  {} {} (priority: {})",
        label("Dynamic Range:").dim(),
        r.dynamic_range,
        r.dynamic_range_priority
    );

    let thirds = (r.exposure * 3.0).round() as i32;
    let exp = match thirds {
        0 => "0".to_string(),
        n => format!("{:+}/3", n),
    };
    println!("  {} {}", label("Exposure:").dim(), exp);
    println!("  {} {}", label("Highlight:").dim(), fmt_f64(r.highlight));
    println!("  {} {}", label("Shadow:").dim(), fmt_f64(r.shadow));
    println!("  {} {}", label("Color:").dim(), fmt_f64(r.color));
    println!("  {} {}", label("Sharpness:").dim(), fmt_f64(r.sharpness));
    println!("  {} {}", label("Clarity:").dim(), fmt_i32(r.clarity));
    println!(
        "  {} {}",
        label("High ISO NR:").dim(),
        fmt_i32(r.high_iso_nr)
    );
}

fn show(args: ShowArgs) -> Result<()> {
    let config = Config::open()?;
    let recipe = match (args.c, args.name) {
        (Some(slot), None) => {
            let mut camera = Camera::open_first()?;
            eprintln!("{camera} Reading C{slot}...");
            camera.read_preset(slot)?
        }
        (None, Some(name)) => config.read_recipe(&name)?,
        _ => anyhow::bail!("specify either -c <slot> or <name>"),
    };
    print_recipe(&recipe);

    Ok(())
}

fn save(args: SaveArgs) -> Result<()> {
    let config = Config::open()?;
    let slot = args.c;
    let mut camera = Camera::open_first()?;

    eprintln!("{camera} Reading C{slot}...");
    let recipe = camera.read_preset(slot)?;
    print_recipe(&recipe);

    let path = config.write_recipe(&args.name, &recipe, args.force)?;
    eprintln!("{} Saved to {}", crate::done(), path.display());

    Ok(())
}

fn load(args: LoadArgs) -> Result<()> {
    let config = Config::open()?;
    let slot = args.c;
    let recipe = config.read_recipe(&args.name)?;
    let mut camera = Camera::open_first()?;

    eprintln!("{camera} Writing C{slot}...");
    print_recipe(&recipe);

    camera.write_preset(slot, &recipe)?;
    eprintln!(
        "{} Loaded {} into C{slot}",
        crate::done(),
        args.name.paint(fujinx::BLUE)
    );

    Ok(())
}

fn del(args: DelArgs) -> Result<()> {
    let config = Config::open()?;
    config.delete_recipe(&args.name)?;
    eprintln!(
        "{} Deleted {}",
        crate::done(),
        args.name.paint(fujinx::BLUE)
    );

    Ok(())
}

fn list() -> Result<()> {
    let config = Config::open()?;
    let recipes = config.list_recipes()?;
    if recipes.is_empty() {
        eprintln!("No recipes found.");
    } else {
        for (name, sources) in &recipes {
            let sources_str: Vec<String> = sources
                .iter()
                .enumerate()
                .map(|(i, source)| {
                    let (label, color) = match source {
                        RecipeSource::Local(s) => (s.as_str(), fujinx::YELLOW),
                        RecipeSource::Remote(s) => (s.as_str(), fujinx::GREEN),
                    };
                    if i == 0 {
                        label.paint(color).bold().to_string()
                    } else {
                        label.paint(color).dim().to_string()
                    }
                })
                .collect();
            println!("{} ➜ {}", name.paint(fujinx::BLUE), sources_str.join(", "));
        }
    }

    Ok(())
}
