use std::process::Command;

use anyhow::{anyhow, Context, Result};
use owo_colors::{OwoColorize, Stream::Stdout};

pub async fn update() -> Result<()> {
    println!(
        "{}",
        "Updating".if_supports_color(Stdout, |t| t.bright_green())
    );

    let config = nix_data::config::configfile::getconfig()?;
    let flakefile = config.flake.context("Failed to get flake file")?;
    let flakearg = config.flakearg;

    let flakestatus = Command::new("sudo")
        .arg("nix")
        .arg("flake")
        .arg("update")
        .arg(&flakefile)
        .status();
    match flakestatus {
        Ok(status) => {
            if !status.success() {
                return Err(anyhow!("Failed to update flake"));
            }
        }
        Err(e) => {
            return Err(anyhow!("Failed to update flake: {}", e));
        }
    }

    let status = Command::new("sudo")
        .arg("nixos-rebuild")
        .arg("switch")
        .arg("--flake")
        .arg(if let Some(arg) = flakearg {
            format!("{}#{}", flakefile, arg)
        } else {
            flakefile
        })
        .status();
    match status {
        Ok(s) if s.success() => {
            println!(
                "{}",
                "Successfully updated system".if_supports_color(Stdout, |t| t.bright_green())
            );
            // Update cache;
            println!(
                "{}",
                "Updating cache".if_supports_color(Stdout, |t| t.bright_yellow())
            );
            let _ = nix_data::cache::flakes::flakespkgs();
            let _ = nix_data::cache::nixos::nixospkgs();
            println!(
                "{}",
                "Cache updated".if_supports_color(Stdout, |t| t.bright_green())
            );
            Ok(())
        }
        _ => {
            eprintln!(
                "{} failed to update",
                "error:".if_supports_color(Stdout, |t| t.bright_red())
            );
            Err(anyhow!("Failed to update system"))
        }
    }
}
