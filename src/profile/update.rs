use std::process::Command;

use anyhow::{anyhow, Result};
use owo_colors::{OwoColorize, Stream::Stdout};

use crate::{PKGSTYLE, VERSIONSTYLE};

pub async fn update(pkg: &str) -> Result<()> {
    let pkgs = nix_data::cache::profile::getprofilepkgs().unwrap();
    let currpkgs = nix_data::cache::profile::getprofilepkgs_versioned()
        .await
        .unwrap();
    if let Some(version) = currpkgs.get(pkg) {
        println!(
            "{} {} ({})",
            "Updating:".if_supports_color(Stdout, |t| t.bright_green()),
            pkg.if_supports_color(Stdout, |t| t.style(*PKGSTYLE)),
            version
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
                    version
                        .as_str()
                        .if_supports_color(Stdout, |t| t.style(*VERSIONSTYLE)),
                );
                Ok(())
            }
            _ => Err(anyhow!(
                "Failed to update {} ({})",
                pkg.if_supports_color(Stdout, |t| t.style(*PKGSTYLE)),
                version
                    .as_str()
                    .if_supports_color(Stdout, |t| t.style(*VERSIONSTYLE))
            )),
        }
    } else if pkgs.get(pkg).map(|x| x.originalurl.to_string())
        == Some(String::from("flake:nixpkgs"))
    {
        println!(
            "{} {}",
            "Updating:".if_supports_color(Stdout, |t| t.bright_green()),
            pkg.if_supports_color(Stdout, |t| t.style(*PKGSTYLE))
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
                    "{} {}",
                    "Successfully updated:".if_supports_color(Stdout, |t| t.bright_green()),
                    pkg.if_supports_color(Stdout, |t| t.style(*PKGSTYLE))
                );
                Ok(())
            }
            _ => Err(anyhow!(
                "Failed to update {}",
                pkg.if_supports_color(Stdout, |t| t.style(*PKGSTYLE))
            )),
        }
    } else {
        let list = Command::new("nix").arg("profile").arg("list").output()?;
        let profilelist = String::from_utf8_lossy(&list.stdout);
        let profilevec = profilelist.split('\n');
        for l in profilevec {
            let parts = l.split(' ').collect::<Vec<&str>>();
            if let Some(p) = parts.get(1) {
                if pkg.eq(*p) {
                    println!(
                        "{} {}",
                        "Updating:".if_supports_color(Stdout, |t| t.bright_green()),
                        pkg.if_supports_color(Stdout, |t| t.style(*PKGSTYLE))
                    );
                    let status = Command::new("nix")
                        .arg("profile")
                        .arg("upgrade")
                        .arg("--impure")
                        // Change to match system
                        .arg(&parts.first().unwrap())
                        .status()?;
                    match status {
                        s if s.success() => {
                            println!(
                                "{} {}",
                                "Successfully updated:"
                                    .if_supports_color(Stdout, |t| t.bright_green()),
                                pkg.if_supports_color(Stdout, |t| t.style(*PKGSTYLE))
                            );
                            return Ok(());
                        }
                        _ => {
                            return Err(anyhow!(
                                "Failed to update {}",
                                pkg.if_supports_color(Stdout, |t| t.style(*PKGSTYLE))
                            ));
                        }
                    }
                }
            }
        }
        // Package is not installed
        eprintln!(
            "{} {} {}",
            "Package".if_supports_color(Stdout, |t| t.bright_yellow()),
            pkg.if_supports_color(Stdout, |t| t.style(*PKGSTYLE)),
            "is not installed on the user profile".if_supports_color(Stdout, |t| t.bright_yellow())
        );
        Err(anyhow!("{} not found in profile", pkg))
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
        _ => Err(anyhow!("Failed to update all user packages")),
    }
}
