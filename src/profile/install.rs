use std::process::Command;

use anyhow::{anyhow, Result};
use owo_colors::{OwoColorize, Stream::Stdout};
use sqlx::SqlitePool;

use crate::{PKGSTYLE, VERSIONSTYLE};

pub async fn install(pkg: &str) -> Result<()> {
    let installed = nix_data::cache::profile::getprofilepkgs()?;
    if installed.contains_key(pkg) {
        println!(
            "{} {}",
            pkg.if_supports_color(Stdout, |t| t.style(*PKGSTYLE)),
            "is already installed".if_supports_color(Stdout, |t| t.bright_yellow()),
        );
        return Ok(());
    }

    let dbfile = nix_data::cache::profile::nixpkgslatest().await?;
    let db = format!("sqlite://{}", dbfile);
    let pool = SqlitePool::connect(&db).await?;
    let p: Result<(String,), sqlx::Error> =
        sqlx::query_as("SELECT version FROM pkgs WHERE attribute LIKE $1")
            .bind(pkg)
            .fetch_one(&pool)
            .await;
    if let Ok((version,)) = p {
        if version.is_empty() {
            println!(
                "{} {}",
                "Installing:".if_supports_color(Stdout, |t| t.bright_green()),
                pkg.if_supports_color(Stdout, |t| t.style(*PKGSTYLE))
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
                        "{} {}",
                        "Successfully installed:".if_supports_color(Stdout, |t| t.bright_green()),
                        pkg.if_supports_color(Stdout, |t| t.style(*PKGSTYLE))
                    );
                    Ok(())
                }
                _ => {
                    eprintln!(
                        "{} failed to install {}",
                        "error:".if_supports_color(Stdout, |t| t.bright_red()),
                        pkg.if_supports_color(Stdout, |t| t.style(*PKGSTYLE)),
                    );
                    Err(anyhow!("Failed to install {}", pkg))
                }
            }
        } else {
            println!(
                "{} {} ({})",
                "Installing:".if_supports_color(Stdout, |t| t.bright_green()),
                pkg.if_supports_color(Stdout, |t| t.style(*PKGSTYLE)),
                version
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
                        version
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
                        version
                            .as_str()
                            .if_supports_color(Stdout, |t| t.style(*VERSIONSTYLE)),
                    );
                    Err(anyhow!("Failed to install {}", pkg))
                }
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
