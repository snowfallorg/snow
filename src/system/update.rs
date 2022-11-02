use std::{process::Command, path::Path};
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
        .arg("update")
        .arg("--flake")
        .arg(&flakefile)
        .arg("--")
        .arg("switch")
        .arg("--flake")
        .arg(if let Some(arg) = flakearg { format!("{}#{}", flakefile, arg) } else { flakefile })
        .arg("--impure")
        .spawn()?;
    writecmd.wait().unwrap();

    let status = writecmd.wait();

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
