use std::{collections::HashMap, fs::File, io::BufReader, process::Command};

use anyhow::{anyhow, Result};
use owo_colors::{OwoColorize, Stream::Stdout};

use crate::{NixPkg, PKGSTYLE, VERSIONSTYLE};

pub fn update(pkg: &str) -> Result<()> {
    let file = nix_data::cache::profile::nixpkgslatest()?;
    let pkgs: HashMap<String, NixPkg> =
        serde_json::from_reader(BufReader::new(File::open(file)?)).unwrap();
    let currpkgs = nix_data::cache::profile::getprofilepkgs_versioned()?;
    if let Some(p) = pkgs.get(pkg) {
        println!(
            "{} {} ({} -> {})",
            "Updating:".if_supports_color(Stdout, |t| t.bright_green()),
            pkg.if_supports_color(Stdout, |t| t.style(*PKGSTYLE)),
            currpkgs
                .get(pkg)
                .unwrap_or(&"unknown".to_string())
                .if_supports_color(Stdout, |t| t.bright_yellow()),
            p.version
                .as_str()
                .if_supports_color(Stdout, |t| t.style(*VERSIONSTYLE)),
        );
        let status = Command::new("nix")
            .arg("profile")
            .arg("upgrade")
            .arg("--impure")
            // Change to match system
            .arg(&format!("legacyPackages.x86_64-linux.{}", pkg))
            .status()?;
        match status {
            s if s.success() => {
                println!(
                    "{} {} ({})",
                    "Successfully updated:".if_supports_color(Stdout, |t| t.bright_green()),
                    pkg.if_supports_color(Stdout, |t| t.style(*PKGSTYLE)),
                    p.version
                        .as_str()
                        .if_supports_color(Stdout, |t| t.style(*VERSIONSTYLE)),
                );
                Ok(())
            }
            _ => {
                eprintln!(
                    "{} failed to update {} ({})",
                    "error:".if_supports_color(Stdout, |t| t.bright_red()),
                    pkg.if_supports_color(Stdout, |t| t.style(*PKGSTYLE)),
                    p.version
                        .as_str()
                        .if_supports_color(Stdout, |t| t.style(*VERSIONSTYLE)),
                );
                Err(anyhow!("Failed to update {}", pkg))
            }
        }
    } else {
        let status = Command::new("nix")
            .arg("profile")
            .arg("update")
            .arg("--impure")
            // Change to match system
            .arg(&format!("legacyPackages.x86_64-linux.{}", pkg))
            .status()?;
        match status {
            s if s.success() => {
                println!(
                    "{} {}",
                    "Successfully updated:".if_supports_color(Stdout, |t| t.bright_green()),
                    pkg.if_supports_color(Stdout, |t| t.style(*PKGSTYLE))
                );
                Ok(())
            }
            _ => {
                eprintln!(
                    "{} failed to update {}",
                    "error:".if_supports_color(Stdout, |t| t.bright_red()),
                    pkg.if_supports_color(Stdout, |t| t.style(*PKGSTYLE))
                );
                Err(anyhow!("Failed to update {}", pkg))
            }
        }
    }
}

pub fn updateall() -> Result<()> {
    println!(
        "{}",
        "Updating all user packages".if_supports_color(Stdout, |t| t.bright_green())
    );
    let status = Command::new("nix")
        .arg("profile")
        .arg("upgrade")
        .arg("--impure")
        .arg(".*")
        .status()?;
    match status {
        s if s.success() => {
            println!(
                "{}",
                "Successfully updated all user packages"
                    .if_supports_color(Stdout, |t| t.bright_green())
            );
            Ok(())
        }
        _ => {
            eprintln!(
                "{} failed to update all user packages",
                "error:".if_supports_color(Stdout, |t| t.bright_red())
            );
            Err(anyhow!("Failed to update all user packages"))
        }
    }
}
