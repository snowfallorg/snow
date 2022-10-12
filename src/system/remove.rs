use std::{
    collections::HashMap,
    fs::{self, File},
    io::BufReader,
    process::Command,
};

use anyhow::{anyhow, Context, Result};
use ijson::{IString, IValue};
use owo_colors::{OwoColorize, Stream::Stdout};

use crate::PKGSTYLE;

pub fn remove(pkgs: &[&str]) -> Result<()> {
    let f = nix_data::cache::flakes::flakespkgs()?;
    let data: HashMap<IString, IValue> =
        serde_json::from_reader(BufReader::new(File::open(f)?)).unwrap();
    for pkg in pkgs {
        if !data.contains_key(&IString::from(pkg.to_string())) {
            eprintln!(
                "{} package {} not found",
                "error:".if_supports_color(Stdout, |t| t.bright_red()),
                pkg.if_supports_color(Stdout, |t| t.style(*PKGSTYLE))
            );
            return Err(anyhow!("Package {} not found", pkg));
        }
    }
    println!(
        "{} {}",
        "Removing:".if_supports_color(Stdout, |t| t.bright_green()),
        pkgs.join(" ")
            .if_supports_color(Stdout, |t| t.style(*PKGSTYLE)),
    );

    let config = nix_data::config::configfile::getconfig()?;
    let configfile = config.systemconfig.context("Failed to get system config")?;
    let flakefile = config.flake.context("Failed to get flake file")?;
    let flakearg = config.flakearg;

    let oldconfig = fs::read_to_string(&configfile)?;
    let currinstalled = nix_data::cache::flakes::getflakepkgs(&[&configfile])?;
    let mut newinstall = vec![];
    for p in pkgs {
        if currinstalled.contains_key(&p.to_string()) {
            newinstall.push(p.to_string());
        }
    }
    if newinstall.is_empty() {
        println!(
            "{}",
            "No packages to remove".if_supports_color(Stdout, |t| t.bright_yellow())
        );
        return Ok(());
    }

    let newconfig = nix_editor::write::rmarr(&oldconfig, "environment.systemPackages", newinstall)?;
    fs::write(&configfile, newconfig)?;
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
                "{} {}",
                "Successfully removed:".if_supports_color(Stdout, |t| t.bright_green()),
                pkgs.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
                    .if_supports_color(Stdout, |t| t.style(*PKGSTYLE)),
            );
            Ok(())
        }
        _ => {
            eprintln!(
                "{} failed to remove {}",
                "error:".if_supports_color(Stdout, |t| t.bright_red()),
                pkgs.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
                    .if_supports_color(Stdout, |t| t.style(*PKGSTYLE)),
            );
            // Restore old config
            fs::write(&configfile, oldconfig)?;
            Err(anyhow!(
                "Failed to remove {}",
                pkgs.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ))
        }
    }
}
