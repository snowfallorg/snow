use std::collections::HashMap;

use anyhow::Result;
use owo_colors::{OwoColorize, Stream::Stdout};
use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use crate::VERSIONSTYLE;

pub async fn search(query: &[&str]) -> Result<()> {
    let dbfile = nix_data::cache::nixos::nixospkgs().await?;
    let db = format!("sqlite://{}", dbfile);
    let pool = SqlitePool::connect(&db).await?;

    let mut queryb: QueryBuilder<Sqlite> = QueryBuilder::new(
        "SELECT pkgs.attribute, description, broken, insecure, unsupported, unfree, version FROM pkgs JOIN meta ON (pkgs.attribute = meta.attribute) WHERE (",
    );
    for (i, q) in query.iter().enumerate() {
        if i == query.len() - 1 {
            queryb
                .push(r#"pkgs.attribute LIKE "#)
                .push_bind(format!("%{}%", q))
                .push(r#" OR description LIKE "#)
                .push_bind(format!("%{}%", q))
                .push(")");
        } else {
            queryb
                .push(r#"pkgs.attribute LIKE "#)
                .push_bind(format!("%{}%", q))
                .push(r#" OR description LIKE "#)
                .push_bind(format!("%{}%", q))
                .push(r#") AND ("#);
        }
    }
    let q: Vec<(String, String, u8, u8, u8, u8, String)> =
        queryb.build_query_as().fetch_all(&pool).await.unwrap();
    let mut outlist = Vec::new();
    for (attr, desc, broken, insecure, unsupported, unfree, version) in q {
        outlist.push((
            attr,
            desc,
            broken != 0,
            insecure != 0,
            unsupported != 0,
            unfree != 0,
            version,
        ));
    }

    let currprofilepkgs = nix_data::cache::profile::getprofilepkgs()?;
    let currsyspkgs = if let Ok(config) = nix_data::config::configfile::getconfig() {
        if let Some(configfile) = config.systemconfig {
            nix_data::cache::flakes::getflakepkgs(&[&configfile]).await
        } else {
            Ok(HashMap::new())
        }
    } else {
        Ok(HashMap::new())
    }?;

    outlist.sort_by(|(apkg, _, _, _, _, _, _), (bpkg, _, _, _, _, _, _)| {
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

    for (pkg, desc, broken, insecure, unsupported, unfree, version) in outlist {
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
        if !version.is_empty() {
            pkg = format!(
                "{} ({})",
                pkg,
                version.if_supports_color(Stdout, |t| t.style(*VERSIONSTYLE))
            );
        }
        if broken {
            pkg = format!(
                "{} ({})",
                pkg,
                "broken".if_supports_color(Stdout, |t| t.bright_red())
            );
        }
        if insecure {
            pkg = format!(
                "{} ({})",
                pkg,
                "insecure".if_supports_color(Stdout, |t| t.bright_red())
            );
        }
        if unsupported {
            pkg = format!(
                "{} ({})",
                pkg,
                "unsupported".if_supports_color(Stdout, |t| t.bright_red())
            );
        }
        if unfree {
            pkg = format!(
                "{} ({})",
                pkg,
                "unfree".if_supports_color(Stdout, |t| t.bright_yellow())
            );
        }
        if !desc.is_empty() {
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
