use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

use anyhow::{anyhow, Context, Result};
use owo_colors::{OwoColorize, Stream::Stdout};
use sqlx::SqlitePool;

use crate::PKGSTYLE;

pub async fn remove(pkgs: &[&str]) -> Result<()> {
    // let f = nix_data::cache::flakes::flakespkgs().await?;
    let dbfile = nix_data::cache::flakes::flakespkgs().await?;
    let db = format!("sqlite://{}", dbfile);
    let pool = SqlitePool::connect(&db).await?;

    // let data: HashMap<IString, IValue> =
    //     serde_json::from_reader(BufReader::new(File::open(f)?)).unwrap();
    let mut removepkgs = Vec::new();
    for pkg in pkgs {
        let p: Result<(String,), sqlx::Error> =
            sqlx::query_as("SELECT attribute FROM pkgs WHERE attribute LIKE $1")
                .bind(pkg)
                .fetch_one(&pool)
                .await;
        if let Ok((_,)) = p {
            removepkgs.push(pkg.to_string());
        } else {
            eprintln!(
                "{} package {} not found",
                "error:".if_supports_color(Stdout, |t| t.bright_red()),
                pkg.if_supports_color(Stdout, |t| t.style(*PKGSTYLE))
            );
        }
    }
    println!(
        "{} {}",
        "Removing:".if_supports_color(Stdout, |t| t.bright_green()),
        removepkgs
            .join(" ")
            .if_supports_color(Stdout, |t| t.style(*PKGSTYLE)),
    );

    let config = nix_data::config::configfile::getconfig()?;
    let configfile = config.systemconfig.context("Failed to get system config")?;
    let flakefile = config.flake.context("Failed to get flake file")?;
    let flakearg = config.flakearg;

    let oldconfig = fs::read_to_string(&configfile)?;
    let currinstalled = nix_data::cache::flakes::getflakepkgs(&[&configfile]).await?;
    let mut newremove = vec![];
    for p in &removepkgs {
        if currinstalled.contains_key(&p.to_string()) {
            newremove.push(p.to_string());
        }
    }
    if newremove.is_empty() {
        println!(
            "{}",
            "No packages to remove".if_supports_color(Stdout, |t| t.bright_yellow())
        );
        return Ok(());
    }

    let envsyspkgs = nix_editor::read::getarrvals(&oldconfig, "environment.systemPackages")?;
    let mut finalremove =  vec![];

    for p in &newremove {
        if envsyspkgs.contains(&p.to_string()) {
            finalremove.push(p.to_string());
        }
        if envsyspkgs.contains(&format!("pkgs.{}", p)) {
            finalremove.push(format!("pkgs.{}", p));
        }
    }

    let newconfig = nix_editor::write::rmarr(&oldconfig, "environment.systemPackages", finalremove)?;

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
                "Successfully removed:".if_supports_color(Stdout, |t| t.bright_green()),
                removepkgs
                    .iter()
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
                removepkgs
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
                    .if_supports_color(Stdout, |t| t.style(*PKGSTYLE)),
            );
            Err(anyhow!(
                "Failed to remove {}",
                removepkgs
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ))
        }
    }
}
