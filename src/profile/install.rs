use std::{collections::HashMap, fs::File, io::BufReader, process::Command};

use anyhow::{anyhow, Result};
use owo_colors::{OwoColorize, Stream::Stdout};

use crate::{NixPkg, PKGSTYLE, VERSIONSTYLE};

pub fn install(pkg: &str) -> Result<()> {
    let file = nix_data::cache::profile::nixpkgslatest()?;
    let pkgs: HashMap<String, NixPkg> =
        serde_json::from_reader(BufReader::new(File::open(file)?)).unwrap();
    if let Some(p) = pkgs.get(pkg) {
        println!(
            "{} {} ({})",
            "Installing:".if_supports_color(Stdout, |t| t.bright_green()),
            pkg.if_supports_color(Stdout, |t| t.style(*PKGSTYLE)),
            p.version
                .as_str()
                .if_supports_color(Stdout, |t| t.style(*VERSIONSTYLE)),
        );
        let status = Command::new("nix")
            .arg("profile")
            .arg("install")
            .arg("--impure")
            .arg(&format!("nixpkgs#{}", pkg))
            .status()?;
        match status {
            s if s.success() => {
                println!(
                    "{} {} ({})",
                    "Successfully installed:".if_supports_color(Stdout, |t| t.bright_green()),
                    pkg.if_supports_color(Stdout, |t| t.style(*PKGSTYLE)),
                    p.version
                        .as_str()
                        .if_supports_color(Stdout, |t| t.style(*VERSIONSTYLE)),
                );
                Ok(())
            }
            _ => {
                eprintln!(
                    "{} failed to install {} ({})",
                    "error:".if_supports_color(Stdout, |t| t.bright_red()),
                    pkg.if_supports_color(Stdout, |t| t.style(*PKGSTYLE)),
                    p.version
                        .as_str()
                        .if_supports_color(Stdout, |t| t.style(*VERSIONSTYLE)),
                );
                Err(anyhow!("Failed to install {}", pkg))
            }
        }
    } else {
        eprintln!(
            "{} package {} not found",
            "error:".if_supports_color(Stdout, |t| t.bright_red()),
            pkg.if_supports_color(Stdout, |t| t.style(*PKGSTYLE))
        );
        Err(anyhow!("Package {} not found", pkg))
    }
}
