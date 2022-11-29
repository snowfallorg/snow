use anyhow::{anyhow, Context, Result};
use owo_colors::{OwoColorize, Stream::Stdout};
use sqlx::SqlitePool;
use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

use crate::{ERRORSTYLE, PKGSTYLE};

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
                "error:".if_supports_color(Stdout, |t| t.style(*ERRORSTYLE)),
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

    let haswithpkgs = if let Ok(withvals) =
        nix_editor::read::getwithvalue(&oldconfig, "environment.systemPackages")
    {
        withvals.contains(&String::from("pkgs"))
    } else {
        false
    };

    let mut newinstall = vec![];
    for p in &installpkgs {
        if !currinstalled.contains_key(&p.to_string()) {
            if haswithpkgs {
                newinstall.push(p.to_string());
            } else {
                newinstall.push(format!("pkgs.{}", p));
            }
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

    let exe = match std::env::current_exe() {
        Ok(mut e) => {
            e.pop(); // root/bin
            e.pop(); // root/
            e.push("libexec"); // root/libexec
            e.push("snow-helper");
            let x = e.to_string_lossy().to_string();
            if Path::new(&x).is_file() {
                x
            } else {
                String::from("snow-helper")
            }
        }
        Err(_) => String::from("snow-helper"),
    };

    let mut writecmd = Command::new("sudo")
        .arg(&exe)
        .arg("config")
        .arg("--output")
        .arg(&configfile)
        .arg("--generations")
        .arg(config.generations.unwrap_or(0).to_string())
        .arg("--")
        .arg("switch")
        .arg("--flake")
        .arg(if let Some(arg) = flakearg {
            format!("{}#{}", flakefile, arg)
        } else {
            flakefile
        })
        .arg("--impure")
        .stdin(Stdio::piped())
        .spawn()?;
    writecmd
        .stdin
        .as_mut()
        .ok_or("stdin not available")
        .unwrap()
        .write_all(newconfig.as_bytes())
        .unwrap();
    writecmd.wait().unwrap();

    let status = writecmd.wait();

    match status {
        Ok(s) if s.success() => {
            println!(
                "{} {}",
                "Successfully installed:".if_supports_color(Stdout, |t| t.bright_green()),
                installpkgs
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
                    .if_supports_color(Stdout, |t| t.style(*PKGSTYLE)),
            );
            Ok(())
        }
        _ => Err(anyhow!(
            "Failed to install {}",
            installpkgs
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(", ")
                .if_supports_color(Stdout, |t| t.style(*PKGSTYLE)),
        )),
    }
}
