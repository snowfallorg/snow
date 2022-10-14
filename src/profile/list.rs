use std::collections::HashMap;

use anyhow::Result;

pub async fn list() -> Result<HashMap<String, String>> {
    let currpkgs = nix_data::cache::profile::getprofilepkgs_versioned().await?;
    let allpkgs = nix_data::cache::profile::getprofilepkgs()?;
    let mut list = HashMap::new();
    for (pkg, prof) in allpkgs {
        if let Some(version) = currpkgs.get(&pkg) {
            list.insert(pkg, version.to_string());
        } else {
            list.insert(pkg, prof.name);
        }
    }
    Ok(list)
}
