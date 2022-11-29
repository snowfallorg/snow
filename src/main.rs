use std::collections::HashMap;
use std::process::{exit, Command, Stdio};

use clap::{ArgGroup, Parser, Subcommand, CommandFactory};
use nix_snow::{profile, system};
use nix_snow::{ERRORSTYLE, VERSIONSTYLE};
use owo_colors::{OwoColorize, Stream::Stdout};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    /// Show version information
    #[clap(short = 'V', long)]
    version: bool,
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
    Run {
        package: String,
        arguments: Vec<String>,
    },
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let cli = Cli::parse();

    if cli.version {
        println!("snow {}", env!("CARGO_PKG_VERSION"));
        let nixcmd = Command::new("nix")
            .arg("--version")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status();
        if let Ok(x) = nixcmd {
            if !x.success() {
                exit(1);
            }
        } else {
            exit(1);
        }
        exit(0);
    }

    if let Some(command) = cli.command {
        match command {
            Commands::Install { packages, system } => {
                if system {
                    let p: Vec<&str> = packages.iter().map(|x| &**x).collect();
                    if let Err(e) = system::install::install(&p).await {
                        eprintln!(
                            "{} {}",
                            "error:".if_supports_color(Stdout, |t| t.style(*ERRORSTYLE)),
                            e
                        );
                        exit(1)
                    }
                } else {
                    for pkg in packages {
                        if let Err(e) = profile::install::install(&pkg).await {
                            eprintln!(
                                "{} {}",
                                "error:".if_supports_color(Stdout, |t| t.style(*ERRORSTYLE)),
                                e
                            );
                            exit(1)
                        }
                    }
                }
            }
            Commands::Remove { packages, system } => {
                if system {
                    let p: Vec<&str> = packages.iter().map(|x| &**x).collect();
                    if let Err(e) = system::remove::remove(&p).await {
                        eprintln!(
                            "{} {}",
                            "error:".if_supports_color(Stdout, |t| t.style(*ERRORSTYLE)),
                            e
                        );
                        exit(1)
                    }
                } else {
                    for pkg in packages {
                        let _ = profile::remove::remove(&pkg).await;
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
                    if let Err(e) = system::update::update().await {
                        eprintln!(
                            "{} {}",
                            "error:".if_supports_color(Stdout, |t| t.style(*ERRORSTYLE)),
                            e
                        );
                        exit(1)
                    }
                    if let Err(e) = profile::update::updateall() {
                        eprintln!(
                            "{} {}",
                            "error:".if_supports_color(Stdout, |t| t.style(*ERRORSTYLE)),
                            e
                        );
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
                    if let Err(e) = system::update::update().await {
                        eprintln!(
                            "{} {}",
                            "error:"
                                .if_supports_color(Stdout, |t| t.bright_red().bold().to_string()),
                            e
                        );
                        exit(1)
                    }
                } else if let Some(pkgs) = packages {
                    for pkg in pkgs {
                        let _ = profile::update::update(&pkg).await;
                    }
                } else if let Err(e) = profile::update::updateall() {
                    eprintln!(
                        "{} {}",
                        "error:".if_supports_color(Stdout, |t| t.style(*ERRORSTYLE)),
                        e
                    );
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
                    let lst = profile::list::list().await;
                    printprofilelist(lst);
                } else if system {
                    let lst = nix_snow::system::list::list().await;
                    printsystemlist(lst);
                } else {
                    let lst = profile::list::list().await;
                    let syslst = nix_snow::system::list::list().await;
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
                    println!(
                        "{} No search query provided",
                        "error:".if_supports_color(Stdout, |t| t.red())
                    );
                    exit(1);
                }
                let query: Vec<&str> = query.iter().map(|x| &**x).collect();
                if let Err(e) = nix_snow::search::search(&query).await {
                    eprintln!(
                        "{} {}",
                        "error:".if_supports_color(Stdout, |t| t.style(*ERRORSTYLE)),
                        e
                    );
                    exit(1)
                };
            }
            Commands::Run { package, arguments } => {
                if let Err(e) = profile::run::run(&package, arguments).await {
                    eprintln!(
                        "{} {}",
                        "error:".if_supports_color(Stdout, |t| t.style(*ERRORSTYLE)),
                        e
                    );
                    exit(1)
                }
            }
        }
    } else {
        let _ = Cli::command().print_help();
    }
}
