use anyhow::Result;
use clap::{Args, Subcommand};
use yansi::Paint;

use fujinx::{Config, RecipeSource};

use crate::progress::Progress;

/// Manage recipe repos.
#[derive(Args)]
pub struct RepoCmd {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Add a recipe repo (git URL).
    Add(AddArgs),

    /// Remove a repo.
    Del(DelArgs),

    /// List repos.
    List,

    /// Update repos.
    Update,
}

#[derive(Args)]
struct AddArgs {
    /// Git repository URL.
    url: String,
}

#[derive(Args)]
struct DelArgs {
    /// Repo name (e.g. github.com/calder/fujixweekly).
    name: String,
}

pub fn repo(args: RepoCmd) -> Result<()> {
    match args.command {
        Command::Add(args) => add(args),
        Command::Del(args) => del(args),
        Command::List => list(),
        Command::Update => update(),
    }
}

fn add(args: AddArgs) -> Result<()> {
    let config = Config::open()?;
    let name = config.add_repo(&args.url)?;
    eprintln!("{} Added repo: {}", crate::done(), name.paint(fujinx::BLUE));

    Ok(())
}

fn del(args: DelArgs) -> Result<()> {
    let config = Config::open()?;
    let name = config.remove_repo(&args.name)?;
    eprintln!(
        "{} Removed repo: {}",
        crate::done(),
        name.paint(fujinx::BLUE)
    );

    Ok(())
}

fn list() -> Result<()> {
    let config = Config::open()?;
    let sources = config.list_recipes_per_source()?;
    if sources.is_empty() {
        eprintln!("No repos found. Add one with: fj repo add <url>");
    } else {
        for (source, names) in &sources {
            let (label, color) = match source {
                RecipeSource::Local(s) => (s.as_str(), fujinx::YELLOW),
                RecipeSource::Remote(s) => (s.as_str(), fujinx::GREEN),
            };
            let suffix = if names.len() == 1 {
                "recipe"
            } else {
                "recipes"
            };
            println!(
                "{} ➜ {} {suffix}",
                label.paint(color),
                names.len().paint(fujinx::BLUE),
            );
            for name in names {
                println!("  {}", name.paint(fujinx::BLUE));
            }
        }
    }

    Ok(())
}

fn update() -> Result<()> {
    let config = Config::open()?;
    let repos = config.list_repos()?;
    let progress = Progress::new(repos.len());
    for repo in &repos {
        eprintln!("{progress} Updating: {}", repo.paint(fujinx::BLUE));
        config.update_repo(repo)?;
    }

    eprintln!(
        "{} Updated {} repos in {}",
        crate::done(),
        repos.len().paint(fujinx::BLUE),
        format!("{:.1}s", progress.elapsed()).paint(fujinx::BLUE)
    );

    Ok(())
}
