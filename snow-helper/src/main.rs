use clap::{self, FromArgMatches, Subcommand};
use std::{
    error::Error,
    fs::{self, File},
    io::{self, Read, Write},
    process::Command,
};

#[derive(Subcommand, Debug)]
enum SubCommands {
    Config {
        /// Write stdin to file in path output
        #[arg(short, long)]
        output: String,

        /// Run `nixos-rebuild` with the given arguments
        arguments: Vec<String>,
    },
    Update {
        /// Path to flake file
        #[arg(short, long)]
        flake: String,

        /// Run `nixos-rebuild` with the given arguments
        arguments: Vec<String>,
    },
}

fn main() {
    let cli = SubCommands::augment_subcommands(clap::Command::new(
        "Helper binary for NixOS Configuration Editor",
    ));
    let matches = cli.get_matches();
    let derived_subcommands = SubCommands::from_arg_matches(&matches)
        .map_err(|err| err.exit())
        .unwrap();

    if users::get_effective_uid() != 0 {
        eprintln!("nixos-conf-editor-helper must be run as root");
        std::process::exit(1);
    }

    match derived_subcommands {
        SubCommands::Config { output, arguments } => {
            match write_file(&output, arguments) {
                Ok(_) => (),
                Err(err) => {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            };
        }
        SubCommands::Update { flake, arguments } => match update(&flake, arguments) {
            Ok(_) => (),
            Err(err) => {
                eprintln!("{}", err);
                std::process::exit(1);
            }
        },
    }
}

fn write_file(path: &str, args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let backup = fs::read_to_string(path)?;

    let stdin = io::stdin();
    let mut buf = String::new();
    stdin.lock().read_to_string(&mut buf)?;
    let mut file = File::create(path)?;
    write!(file, "{}", &buf)?;

    let mut cmd = Command::new("nixos-rebuild").args(args).spawn()?;
    if let Ok(x) = cmd.wait() {
        if x.success() {
            Ok(())
        } else {
            let mut file = File::create(path)?;
            write!(file, "{}", &backup)?;
            eprintln!("nixos-rebuild failed with exit code {}", x.code().unwrap());
            std::process::exit(1);
        }
    } else {
        let mut file = File::create(path)?;
        write!(file, "{}", &backup)?;
        eprintln!("nixos-rebuild failed");
        std::process::exit(1);
    }
}

fn update(path: &str, args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::new("nix")
        .arg("flake")
        .arg("update")
        .arg(&path)
        .spawn()?;
    let x = cmd.wait()?;
    if !x.success() {
        eprintln!(
            "nix flake update failed with exit code {}",
            x.code().unwrap()
        );
        std::process::exit(1);
    }

    let mut cmd = Command::new("nixos-rebuild").args(args).spawn()?;
    let x = cmd.wait()?;
    if x.success() {
        Ok(())
    } else {
        eprintln!("nixos-rebuild failed with exit code {}", x.code().unwrap());
        std::process::exit(1);
    }
}
