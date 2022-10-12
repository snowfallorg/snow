use std::{collections::HashMap, fs::File, io::BufReader};

use anyhow::Result;
use ijson::IString;
use owo_colors::{OwoColorize, Stream::Stdout};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct NixPkgList {
    packages: HashMap<IString, NixPkg>,
}

#[derive(Debug, Deserialize)]
struct NixPkg {
    meta: NixPkgMeta,
}

#[derive(Debug, Deserialize)]
struct NixPkgMeta {
    broken: Option<bool>,
    insecure: Option<bool>,
    unsupported: Option<bool>,
    unfree: Option<bool>,
    description: Option<IString>,
}

pub fn search(query: &[&str]) -> Result<()> {
    let file = nix_data::cache::nixos::nixospkgs()?;
    let pkgs: NixPkgList = serde_json::from_reader(BufReader::new(File::open(file)?)).unwrap();
    let currprofilepkgs = nix_data::cache::profile::getprofilepkgs()?;
    let currsyspkgs = if let Ok(config) = nix_data::config::configfile::getconfig() {
        if let Some(configfile) = config.systemconfig {
            nix_data::cache::flakes::getflakepkgs(&[&configfile])
        } else {
            Ok(HashMap::new())
        }
    } else {
        Ok(HashMap::new())
    }?;

    let mut outlist = Vec::new();
    for (pkg, data) in pkgs.packages {
        if query
            .iter()
            .any(|x| pkg.to_lowercase().contains(&x.to_lowercase()))
        {
            outlist.push((pkg.to_string(), data.meta));
        } else if let Some(desc) = &data.meta.description {
            if query
                .iter()
                .any(|x| desc.to_lowercase().contains(&x.to_lowercase()))
            {
                outlist.push((pkg.to_string(), data.meta));
            }
        }
    }

    outlist.sort_by(|(apkg, _), (bpkg, _)| {
        let mut aleft = apkg.to_lowercase();
        let mut bleft = bpkg.to_lowercase();
        for q in query {
            let q = &q.to_lowercase();
            if aleft.contains(q) {
                aleft = aleft.replace(q, "");
            } else {
                aleft.push_str(q);
            }
            if bleft.contains(q) {
                bleft = bleft.replace(q, "");
            } else {
                bleft.push_str(q);
            }
        }
        bleft.len().cmp(&aleft.len())
    });

    for (pkg, data) in outlist {
        let p = pkg.to_string();
        let mut pkg = p
            .if_supports_color(Stdout, |t| {
                let mut t = format!("{}", t.bold());
                for q in query {
                    let qlower = q.to_lowercase();
                    let tlower = t.to_lowercase();
                    if tlower.contains(&qlower) {
                        let m = tlower.match_indices(&qlower);
                        let mut off = 0;
                        for (i, s) in m {
                            t.replace_range(
                                i + off..i + off + s.len(),
                                t[i + off..i + off + s.len()]
                                    .to_string()
                                    .bright_green()
                                    .to_string()
                                    .as_str(),
                            );
                            off += 10;
                        }
                    }
                }
                t
            })
            .to_string();
        if currprofilepkgs.contains_key(&p) {
            pkg = format!("{} ({})", pkg, "user".bright_cyan());
        }
        if currsyspkgs.contains_key(&p) {
            pkg = format!("{} ({})", pkg, "system".bright_magenta());
        }
        if let Some(broken) = data.broken {
            if broken {
                pkg = format!(
                    "{} ({})",
                    pkg,
                    "broken".if_supports_color(Stdout, |t| t.bright_red())
                );
            }
        }
        if let Some(insecure) = data.insecure {
            if insecure {
                pkg = format!(
                    "{} ({})",
                    pkg,
                    "insecure".if_supports_color(Stdout, |t| t.bright_red())
                );
            }
        }
        if let Some(unsupported) = data.unsupported {
            if unsupported {
                pkg = format!(
                    "{} ({})",
                    pkg,
                    "unsupported".if_supports_color(Stdout, |t| t.bright_red())
                );
            }
        }
        if let Some(unfree) = data.unfree {
            if unfree {
                pkg = format!(
                    "{} ({})",
                    pkg,
                    "unfree".if_supports_color(Stdout, |t| t.bright_yellow())
                );
            }
        }
        if let Some(desc) = data.description {
            println!(
                "* {}\n  {}\n",
                pkg,
                desc.as_str().if_supports_color(Stdout, |t| {
                    let mut t = t.to_string();
                    for q in query {
                        let qlower = q.to_lowercase();
                        let tlower = t.to_lowercase();
                        if tlower.contains(&qlower) {
                            let m = tlower.match_indices(&qlower);
                            let mut off = 0;
                            for (i, s) in m {
                                t.replace_range(
                                    i + off..i + off + s.len(),
                                    t[i + off..i + off + s.len()]
                                        .to_string()
                                        .bright_green()
                                        .to_string()
                                        .as_str(),
                                );
                                off += 10;
                            }
                        }
                    }
                    t
                })
            );
        } else {
            println!("* {}\n", pkg);
        }
    }
    Ok(())
}
