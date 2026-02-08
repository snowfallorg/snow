use clap::{ArgGroup, CommandFactory, Parser, Subcommand};
use nix_snow::{
    ERRORSTYLE, VERSIONSTYLE, WARNINGSTYLE, is_home_configured, is_profile_configured,
    is_system_configured,
};
use owo_colors::{OwoColorize, Stream::Stdout};
use std::{
    path::Path,
    process::{Command, Stdio, exit},
};

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
    #[command(group(ArgGroup::new("install").args(&["system", "home"])))]
    Install {
        packages: Vec<String>,
        #[arg(short, long)]
        system: bool,
        #[arg(short, long)]
        home: bool,
    },
    #[command(group(ArgGroup::new("remove").args(&["system", "home"])))]
    Remove {
        packages: Vec<String>,
        #[arg(short, long)]
        system: bool,
        #[arg(short, long)]
        home: bool,
    },
    #[command(group(ArgGroup::new("update").args(&["system", "home", "all"])))]
    Update {
        packages: Option<Vec<String>>,
        #[arg(short, long)]
        system: bool,
        #[arg(short, long)]
        home: bool,
        #[arg(short, long)]
        all: bool,
    },
    #[command(group(ArgGroup::new("rebuild").args(&["system", "home"])))]
    Rebuild {
        #[arg(short, long)]
        system: bool,
        #[arg(short, long)]
        home: bool,
    },
    #[command(group(ArgGroup::new("listtype").args(&["profile", "system", "home"])))]
    List {
        #[arg(short, long)]
        profile: bool,
        #[arg(short, long)]
        system: bool,
        #[arg(short, long)]
        home: bool,
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
            Commands::Install {
                packages,
                system,
                home,
            } => {
                if system {
                    let p: Vec<&str> = packages.iter().map(|x| &**x).collect();
                    let md = libsnow::metadata::Metadata::connect().await.unwrap();
                    if let Err(e) =
                        libsnow::nixos::install::install(&p, &md, libsnow::nixos::AuthMethod::Sudo)
                            .await
                    {
                        eprintln!(
                            "{} {}",
                            "error:".if_supports_color(Stdout, |t| t.style(*ERRORSTYLE)),
                            e
                        );
                        exit(1)
                    }
                } else if home {
                    check_home_manager();
                    let p: Vec<&str> = packages.iter().map(|x| &**x).collect();
                    let md = libsnow::metadata::Metadata::connect().await.unwrap();
                    if let Err(e) = libsnow::homemanager::install::install(&p, &md).await {
                        eprintln!(
                            "{} {}",
                            "error:".if_supports_color(Stdout, |t| t.style(*ERRORSTYLE)),
                            e
                        );
                        exit(1)
                    }
                } else {
                    let p: Vec<&str> = packages.iter().map(|x| &**x).collect();
                    if let Err(e) = libsnow::profile::install::install(&p).await {
                        eprintln!(
                            "{} {}",
                            "error:".if_supports_color(Stdout, |t| t.style(*ERRORSTYLE)),
                            e
                        );
                        exit(1)
                    }
                }
                if let Err(e) = libsnow::utils::misc::refresh_icons() {
                    eprintln!(
                        "{} failed to refresh icons: {}",
                        "warning:".if_supports_color(Stdout, |t| t.style(*WARNINGSTYLE)),
                        e
                    );
                }
            }
            Commands::Remove {
                packages,
                system,
                home,
            } => {
                if system {
                    let p: Vec<&str> = packages.iter().map(|x| &**x).collect();
                    let md = libsnow::metadata::Metadata::connect().await.unwrap();
                    if let Err(e) =
                        libsnow::nixos::remove::remove(&p, &md, libsnow::nixos::AuthMethod::Sudo)
                            .await
                    {
                        eprintln!(
                            "{} {}",
                            "error:".if_supports_color(Stdout, |t| t.style(*ERRORSTYLE)),
                            e
                        );
                        exit(1)
                    }
                } else if home {
                    check_home_manager();
                    let p: Vec<&str> = packages.iter().map(|x| &**x).collect();
                    let md = libsnow::metadata::Metadata::connect().await.unwrap();
                    if let Err(e) = libsnow::homemanager::remove::remove(&p, &md).await {
                        eprintln!(
                            "{} {}",
                            "error:".if_supports_color(Stdout, |t| t.style(*ERRORSTYLE)),
                            e
                        );
                        exit(1)
                    }
                } else {
                    let p: Vec<&str> = packages.iter().map(|x| &**x).collect();
                    if let Err(e) = libsnow::profile::remove::remove(&p).await {
                        eprintln!(
                            "{} {}",
                            "error:".if_supports_color(Stdout, |t| t.style(*ERRORSTYLE)),
                            e
                        );
                        exit(1)
                    }
                }
                if let Err(e) = libsnow::utils::misc::refresh_icons() {
                    eprintln!(
                        "{} failed to refresh icons: {}",
                        "warning:".if_supports_color(Stdout, |t| t.style(*WARNINGSTYLE)),
                        e
                    );
                }
            }
            Commands::Update {
                packages,
                system,
                home,
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
                    if is_system_configured()
                        && let Err(e) =
                            libsnow::nixos::update::update(libsnow::nixos::AuthMethod::Sudo).await
                    {
                        eprintln!(
                            "{} {}",
                            "error:".if_supports_color(Stdout, |t| t.style(*ERRORSTYLE)),
                            e
                        );
                        exit(1)
                    }
                    if is_profile_configured()
                        && let Err(e) = libsnow::profile::update::update_all().await
                    {
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
                    if let Err(e) =
                        libsnow::nixos::update::update(libsnow::nixos::AuthMethod::Sudo).await
                    {
                        eprintln!(
                            "{} {}",
                            "error:"
                                .if_supports_color(Stdout, |t| t.bright_red().bold().to_string()),
                            e
                        );
                        exit(1)
                    }
                } else if home {
                    check_home_manager();
                    if packages.is_some() {
                        println!(
                            "{} ignoring packages passed to home-manager upgrade",
                            "warning:".if_supports_color(Stdout, |t| t.bright_yellow())
                        );
                    }
                    if let Err(e) = libsnow::homemanager::update::update().await {
                        eprintln!(
                            "{} {}",
                            "error:"
                                .if_supports_color(Stdout, |t| t.bright_red().bold().to_string()),
                            e
                        );
                        exit(1)
                    }
                } else if let Some(pkgs) = packages {
                    let _ = libsnow::profile::update::update(
                        &pkgs.iter().map(|x| x.as_str()).collect::<Vec<_>>(),
                    )
                    .await;
                } else if let Err(e) = libsnow::profile::update::update_all().await {
                    eprintln!(
                        "{} {}",
                        "error:".if_supports_color(Stdout, |t| t.style(*ERRORSTYLE)),
                        e
                    );
                    exit(1)
                }
                if let Err(e) = libsnow::utils::misc::refresh_icons() {
                    eprintln!(
                        "{} failed to refresh icons: {}",
                        "warning:".if_supports_color(Stdout, |t| t.style(*WARNINGSTYLE)),
                        e
                    );
                }
            }
            Commands::Rebuild { system, home } => {
                if system || !home {
                    if let Err(e) =
                        libsnow::nixos::rebuild::rebuild(libsnow::nixos::AuthMethod::Sudo).await
                    {
                        eprintln!(
                            "{} {}",
                            "error:".if_supports_color(Stdout, |t| t.style(*ERRORSTYLE)),
                            e
                        );
                        exit(1)
                    }
                } else if home {
                    check_home_manager();
                    if let Err(e) = libsnow::homemanager::rebuild::rebuild().await {
                        eprintln!(
                            "{} {}",
                            "error:".if_supports_color(Stdout, |t| t.style(*ERRORSTYLE)),
                            e
                        );
                        exit(1)
                    }
                }
            }
            Commands::List {
                profile,
                system,
                home,
            } => {
                fn printprofilelist(mut lst: Vec<libsnow::Package>) {
                    lst.sort_by(|a, b| a.attr.to_string().cmp(&b.attr.to_string()));
                    println!(
                        "{}",
                        "Profile Packages:".if_supports_color(Stdout, |t| t.bright_cyan())
                    );
                    for pkg in lst {
                        println!(
                            "{} ({})",
                            pkg.attr,
                            pkg.version
                                .unwrap_or_default()
                                .if_supports_color(Stdout, |t| t.style(*VERSIONSTYLE))
                        );
                    }
                }
                fn printsystemlist(mut lst: Vec<libsnow::Package>) {
                    lst.sort_by(|a, b| a.attr.to_string().cmp(&b.attr.to_string()));
                    println!(
                        "{}",
                        "System Packages:".if_supports_color(Stdout, |t| t.bright_cyan())
                    );
                    for pkg in lst {
                        if let Some(v) = pkg.version {
                            println!(
                                "{} ({})",
                                pkg.attr,
                                v.if_supports_color(Stdout, |t| t.style(*VERSIONSTYLE))
                            );
                        } else {
                            println!("{}", pkg.attr);
                        }
                    }
                }
                fn printhomelist(mut lst: Vec<libsnow::Package>) {
                    lst.sort_by(|a, b| a.attr.to_string().cmp(&b.attr.to_string()));
                    println!(
                        "{}",
                        "Home Manager Packages:".if_supports_color(Stdout, |t| t.bright_cyan())
                    );
                    for pkg in lst {
                        println!(
                            "{} ({})",
                            pkg.attr,
                            pkg.version
                                .unwrap_or_default()
                                .if_supports_color(Stdout, |t| t.style(*VERSIONSTYLE))
                        );
                    }
                }
                if profile {
                    let lst = libsnow::profile::list::list();
                    match lst {
                        Ok(lst) => printprofilelist(lst),
                        Err(e) => {
                            eprintln!(
                                "{} {}",
                                "error:".if_supports_color(Stdout, |t| t.style(*ERRORSTYLE)),
                                e
                            );
                            exit(1);
                        }
                    }
                } else if system {
                    let md = libsnow::metadata::Metadata::connect().await.unwrap();
                    let lst = libsnow::nixos::list::list_systempackages(&md);
                    match lst {
                        Ok(lst) => printsystemlist(lst),
                        Err(e) => {
                            eprintln!(
                                "{} {}",
                                "error:".if_supports_color(Stdout, |t| t.style(*ERRORSTYLE)),
                                e
                            );
                            exit(1);
                        }
                    }
                } else if home {
                    check_home_manager();
                    let md = libsnow::metadata::Metadata::connect().await.unwrap();
                    let lst = libsnow::homemanager::list::list(&md);
                    match lst {
                        Ok(lst) => printhomelist(lst),
                        Err(e) => {
                            eprintln!(
                                "{} {}",
                                "error:".if_supports_color(Stdout, |t| t.style(*ERRORSTYLE)),
                                e
                            );
                            exit(1);
                        }
                    }
                } else {
                    let md = libsnow::metadata::Metadata::connect().await.unwrap();
                    let mut printed_first = false;
                    if is_profile_configured() {
                        let lst = libsnow::profile::list::list();
                        match lst {
                            Ok(lst) => {
                                printprofilelist(lst);
                                printed_first = true;
                            }
                            Err(e) => {
                                eprintln!(
                                    "{} {}",
                                    "error:".if_supports_color(Stdout, |t| t.style(*ERRORSTYLE)),
                                    e
                                );
                                exit(1);
                            }
                        }
                    }
                    if is_system_configured() {
                        let syslst = libsnow::nixos::list::list_systempackages(&md);
                        match syslst {
                            Ok(lst) => {
                                if printed_first {
                                    println!();
                                } else {
                                    printed_first = true;
                                }
                                printsystemlist(lst);
                            }
                            Err(e) => {
                                eprintln!(
                                    "{} {}",
                                    "error:".if_supports_color(Stdout, |t| t.style(*ERRORSTYLE)),
                                    e
                                );
                                exit(1);
                            }
                        }
                    }
                    if home_manager_installed() && is_home_configured() {
                        let homelst = libsnow::homemanager::list::list(&md);
                        match homelst {
                            Ok(homelst) => {
                                if printed_first {
                                    println!();
                                }
                                printhomelist(homelst);
                            }
                            Err(e) => {
                                eprintln!(
                                    "{} {}",
                                    "error:".if_supports_color(Stdout, |t| t.style(*ERRORSTYLE)),
                                    e
                                );
                                exit(1);
                            }
                        }
                    }
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
                if let Err(e) = libsnow::profile::run::run(
                    &package,
                    &arguments.iter().map(|x| x.as_str()).collect::<Vec<_>>(),
                )
                .await
                {
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

fn check_home_manager() {
    if !home_manager_installed() {
        eprintln!(
            "{} Home Manager is not installed. Please install it first.",
            "error:".if_supports_color(Stdout, |t| t.style(*ERRORSTYLE))
        );
        exit(1);
    }
}

fn home_manager_installed() -> bool {
    Path::new(&format!(
        "{}/.local/state/nix/profiles/home-manager",
        std::env::var("HOME").unwrap().as_str()
    ))
    .is_symlink()
}
