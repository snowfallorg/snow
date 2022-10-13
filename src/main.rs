use std::collections::HashMap;
use std::process::exit;

use clap::{ArgGroup, Parser, Subcommand};
use nix_snow::VERSIONSTYLE;
use nix_snow::{profile, system};
use owo_colors::{OwoColorize, Stream::Stdout};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Install {
        packages: Vec<String>,
        #[arg(short, long)]
        system: bool,
    },
    Remove {
        packages: Vec<String>,
        #[arg(short, long)]
        system: bool,
    },
    #[command(group(ArgGroup::new("install").args(&["system", "all"])))]
    Update {
        packages: Option<Vec<String>>,
        #[arg(short, long)]
        system: bool,
        #[arg(short, long)]
        all: bool,
    },
    #[command(group(ArgGroup::new("listtype").args(&["profile", "system"])))]
    List {
        #[arg(short, long)]
        profile: bool,

        #[arg(short, long)]
        system: bool,
    },
    Search {
        query: Vec<String>,
    },
}

fn main() {
    pretty_env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Install { packages, system } => {
            if system {
                let p: Vec<&str> = packages.iter().map(|x| &**x).collect();
                if system::install::install(&p).is_err() {
                    exit(1)
                }
            } else {
                for pkg in packages {
                    if profile::install::install(&pkg).is_err() {
                        exit(1)
                    }
                }
            }
        }
        Commands::Remove { packages, system } => {
            if system {
                let p: Vec<&str> = packages.iter().map(|x| &**x).collect();
                if system::remove::remove(&p).is_err() {
                    exit(1)
                }
            } else {
                for pkg in packages {
                    let _ = profile::remove::remove(&pkg);
                }
            }
        }
        Commands::Update {
            packages,
            system,
            all,
        } => {
            if all {
                // System upgrade updates all packages
                if packages.is_some() {
                    println!(
                        "{} ignoring packages passed to full upgrade",
                        "warning:".if_supports_color(Stdout, |t| t.bright_yellow())
                    );
                }
                if system::update::update().is_err() {
                    exit(1)
                }
                if profile::update::updateall().is_err() {
                    exit(1)
                }
            } else if system {
                // System upgrade updates all packages
                if packages.is_some() {
                    println!(
                        "{} ignoring packages passed to system upgrade",
                        "warning:".if_supports_color(Stdout, |t| t.bright_yellow())
                    );
                }
                if system::update::update().is_err() {
                    exit(1)
                }
            } else if let Some(pkgs) = packages {
                for pkg in pkgs {
                    let _ = profile::update::update(&pkg);
                }
            } else if profile::update::updateall().is_err() {
                exit(1)
            }
        }
        Commands::List { profile, system } => {
            fn printprofilelist(lst: Result<HashMap<String, String>, anyhow::Error>) {
                if let Ok(pkgs) = lst {
                    let mut list = pkgs.into_iter().collect::<Vec<_>>();
                    list.sort();
                    for (pkg, version) in list {
                        println!(
                            "{} ({})",
                            pkg,
                            version.if_supports_color(Stdout, |t| t.style(*VERSIONSTYLE))
                        );
                    }
                } else {
                    exit(1);
                }
            }
            fn printsystemlist(lst: Result<HashMap<String, Option<String>>, anyhow::Error>) {
                if let Ok(pkgs) = lst {
                    let mut list = pkgs.into_iter().collect::<Vec<_>>();
                    list.sort();
                    for (pkg, version) in list {
                        if let Some(v) = version {
                            println!(
                                "{} ({})",
                                pkg,
                                v.if_supports_color(Stdout, |t| t.style(*VERSIONSTYLE))
                            );
                        } else {
                            println!("{}", pkg);
                        }
                    }
                } else {
                    exit(1);
                }
            }
            if profile {
                let lst = profile::list::list();
                printprofilelist(lst);
            } else if system {
                let lst = nix_snow::system::list::list();
                printsystemlist(lst);
            } else {
                let lst = profile::list::list();
                let syslst = nix_snow::system::list::list();
                println!(
                    "{}",
                    "Profile Packages:".if_supports_color(Stdout, |t| t.bright_cyan())
                );
                printprofilelist(lst);
                println!();
                println!(
                    "{}",
                    "System Packages:".if_supports_color(Stdout, |t| t.bright_cyan())
                );
                printsystemlist(syslst);
            }
        }
        Commands::Search { query } => {
            if query.is_empty() {
                println!("{} No search query provided", "error:".if_supports_color(Stdout, |t| t.red()));
                exit(1);
            }
            let query: Vec<&str> = query.iter().map(|x| &**x).collect();
            if nix_snow::search::search(&query).is_err() {
                exit(1)
            };
        }
    }
}
