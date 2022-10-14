use std::{
    process::Command, fs,
};
use anyhow::{anyhow, Context, Result};
use owo_colors::{OwoColorize, Stream::Stdout};
use sqlx::SqlitePool;

use crate::PKGSTYLE;

pub async fn install(pkgs: &[&str]) -> Result<()> {
    let dbfile = nix_data::cache::flakes::flakespkgs().await?;
    let db = format!("sqlite://{}", dbfile);
    let pool = SqlitePool::connect(&db).await?;

    let mut installpkgs = Vec::new();
    for pkg in pkgs {
        let p: Result<(String,), sqlx::Error> =
            sqlx::query_as("SELECT attribute FROM pkgs WHERE attribute LIKE $1")
                .bind(pkg)
                .fetch_one(&pool)
                .await;
        if let Ok((_,)) = p {
            installpkgs.push(pkg.to_string());
        } else {
            eprintln!(
                "{} package {} not found",
                "error:".if_supports_color(Stdout, |t| t.bright_red()),
                pkg.if_supports_color(Stdout, |t| t.style(*PKGSTYLE))
            );
        }
    }

    if installpkgs.is_empty() {
        return Err(anyhow!("No packages found"));
    }

    println!(
        "{} {}",
        "Installing:".if_supports_color(Stdout, |t| t.bright_green()),
        installpkgs
            .join(" ")
            .if_supports_color(Stdout, |t| t.style(*PKGSTYLE)),
    );

    let config = nix_data::config::configfile::getconfig()?;
    let configfile = config.systemconfig.context("Failed to get system config")?;
    let flakefile = config.flake.context("Failed to get flake file")?;
    let flakearg = config.flakearg;

    let oldconfig = fs::read_to_string(&configfile)?;
    let currinstalled = nix_data::cache::flakes::getflakepkgs(&[&configfile]).await?;
    let mut newinstall = vec![];
    for p in &installpkgs {
        if !currinstalled.contains_key(&p.to_string()) {
            newinstall.push(p.to_string());
        }
    }
    if newinstall.is_empty() {
        println!(
            "{}",
            "All packages are already installed".if_supports_color(Stdout, |t| t.bright_green())
        );
        return Ok(());
    }

    let newconfig =
        nix_editor::write::addtoarr(&oldconfig, "environment.systemPackages", newinstall)?;
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
                "Successfully installed:".if_supports_color(Stdout, |t| t.bright_green()),
                installpkgs.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
                    .if_supports_color(Stdout, |t| t.style(*PKGSTYLE)),
            );
            Ok(())
        }
        _ => {
            eprintln!(
                "{} failed to install {}",
                "error:".if_supports_color(Stdout, |t| t.bright_red()),
                installpkgs.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
                    .if_supports_color(Stdout, |t| t.style(*PKGSTYLE)),
            );
            // Restore old config
            fs::write(&configfile, oldconfig)?;
            Err(anyhow!(
                "Failed to install {}",
                installpkgs.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ))
        }
    }
}
