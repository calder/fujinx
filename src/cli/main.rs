mod camera;
mod convert;
mod help;
mod parse;
mod progress;
mod recipe;
mod repo;

use clap::Parser;
use clap::Subcommand;
use yansi::Paint;

use crate::help::{BANNER, STYLES};

/// Fujifilm X Series camera tools.
#[derive(Parser)]
#[command(styles = STYLES, before_help = BANNER)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Camera(camera::CameraCmd),
    Convert(convert::ConvertCmd),
    Recipe(recipe::RecipeCmd),
    Repo(repo::RepoCmd),
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Camera(args) => camera::camera(args),
        Command::Convert(args) => convert::convert(args),
        Command::Recipe(args) => recipe::recipe(args),
        Command::Repo(args) => repo::repo(args),
    };
    if let Err(e) = result {
        let msg = e.to_string();
        let msg = capitalize(&msg);
        let msg = if msg.ends_with('.') { msg } else { msg + "." };
        eprintln!("{} {msg}", "Error:".paint(fujinx::RED).bold());
        std::process::exit(1);
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}

pub fn done() -> yansi::Painted<&'static str> {
    "Done:".paint(fujinx::GREEN).bold()
}
