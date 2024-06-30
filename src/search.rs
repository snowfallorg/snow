use crate::VERSIONSTYLE;
use anyhow::Result;
use libsnow::metadata::search::{get_searcher, SearchQuery, SearchResult};
use owo_colors::{OwoColorize, Stream::Stdout};

pub async fn search(query: &[&str]) -> Result<()> {
    let db = libsnow::metadata::database::database_connection().await?;

    let currprofilepkgs = libsnow::profile::list::list()?
        .into_iter()
        .map(|x| x.attr.to_string())
        .collect::<Vec<_>>();
    let currsyspkgs = libsnow::nixos::list::list_systempackages(&db)?
        .into_iter()
        .map(|x| x.attr.to_string())
        .collect::<Vec<_>>();

    let dbsearcher = get_searcher(&db)?;
    let mut search_result = libsnow::metadata::search::search(
        &SearchQuery {
            query: &query.join(" "),
            limit: 100,
            score_threshold: 0.0,
        },
        &dbsearcher,
    )?;
    search_result.reverse();

    for SearchResult {
        attribute,
        version,
        pname: _,
        description,
        broken,
        insecure,
        unfree,
        score: _,
    } in search_result
    {
        let mut pkg = attribute
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
        if currprofilepkgs.contains(&attribute) {
            pkg = format!("{} ({})", pkg, "user".bright_cyan());
        }
        if currsyspkgs.contains(&attribute) {
            pkg = format!("{} ({})", pkg, "system".bright_magenta());
        }
        if let Some(version) = version {
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
        if unfree {
            pkg = format!(
                "{} ({})",
                pkg,
                "unfree".if_supports_color(Stdout, |t| t.bright_yellow())
            );
        }
        if let Some(description) = description {
            println!(
                "* {}\n  {}\n",
                pkg,
                description.as_str().if_supports_color(Stdout, |t| {
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
