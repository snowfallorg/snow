use anyhow::{anyhow, Result};
use sqlx::SqlitePool;
use std::process::Command;

pub async fn run(pkg: &str, arguments: Vec<String>) -> Result<()> {
    let dbfile = nix_data::cache::profile::nixpkgslatest().await?;
    let db = format!("sqlite://{}", dbfile);
    let pool = SqlitePool::connect(&db).await?;

    // Check if package is in nixpkgs
    let p: Result<(String,), sqlx::Error> =
        sqlx::query_as("SELECT attribute FROM pkgs WHERE attribute = $1")
            .bind(pkg)
            .fetch_one(&pool)
            .await;

    if let Ok((pkg,)) = p {
        let status = Command::new("nix")
            .arg("run")
            .arg("--impure")
            .arg(&format!("nixpkgs#{}", pkg))
            .arg("--")
            .args(arguments)
            .status()?;
        if !status.success() {
            return Err(anyhow!("Failed to run {}", pkg));
        }
    } else {
        let status = Command::new("nix")
            .arg("run")
            .arg("--impure")
            .arg(pkg)
            .arg("--")
            .args(arguments)
            .status()?;
        if !status.success() {
            return Err(anyhow!("Failed to run {}", pkg));
        }
    }
    Ok(())
}
