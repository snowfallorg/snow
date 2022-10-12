use std::{collections::HashMap, fs};

use anyhow::{Context, Result};

pub fn list() -> Result<HashMap<String, Option<String>>> {
    let config = nix_data::config::configfile::getconfig()?;
    let configfile = config.systemconfig.context("Failed to get config file")?;
    let currpkgs = nix_data::cache::flakes::getflakepkgs(&[&configfile])?;
    let allpkgs = nix_editor::read::getarrvals(
        &fs::read_to_string(configfile)?,
        "environment.systemPackages",
    )?;
    let mut list = HashMap::new();
    for pkg in allpkgs {
        if let Some(version) = currpkgs.get(&pkg) {
            list.insert(pkg, Some(version.to_string()));
        } else {
            list.insert(pkg, None);
        }
    }
    Ok(list)
}
