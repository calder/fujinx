use std::process;

use anyhow::Result;
use clap::{Args, Subcommand};
use yansi::Paint;

use fujinx::{Camera, RED};

/// Manage connected cameras.
#[derive(Args)]
pub struct CameraCmd {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// List connected cameras.
    List,
}

pub fn camera(args: CameraCmd) -> Result<()> {
    match args.command {
        Command::List => list(),
    }
}

fn list() -> Result<()> {
    let cameras = Camera::detect()?;
    if cameras.is_empty() {
        eprintln!("{}", "No cameras found.".paint(RED).bold());
        process::exit(1);
    }

    for camera in &cameras {
        let info = camera.info();
        println!("{camera}");
        println!("  {} {}", "Manufacturer:".dim(), info.manufacturer);
        println!("  {}      {}", "Version:".dim(), info.device_version);
        println!("  {}       {}", "Serial:".dim(), info.serial_number);
    }

    Ok(())
}
